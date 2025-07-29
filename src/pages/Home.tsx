import { useState, useEffect } from 'react';
import {AllayLayout} from "../components/common/AllayLayout.tsx";
import {ActionBar} from "../components/common/ActionBar.tsx";
import { DraggableServerList } from "../components/server/DraggableServerList.tsx";
import { ScrollContainer } from "../components/common/ScrollContainer.tsx";
import { EditServerModal } from "../components/modals/EditServerModal.tsx";
import { DeleteServerModal } from "../components/modals/DeleteServerModal.tsx";
import Settings from "./Settings.tsx";
import ServerDetails from "./ServerDetails.tsx";
import { invoke } from '@tauri-apps/api/core';
import { useLocale } from '../contexts/LocaleContext';

interface Server {
    id: string;
    name: string;
    description: string;
    hasCustomImg: boolean;
    imgUrl: string;
    version: string;
    serverType: string;
    loaderVersion: string;
    isOnline: boolean;
    playerCount: number;
    maxPlayers: number;
    memory?: number; // Memory in MB
}

interface ServerInstance {
    name: string;
    version: string;
    mod_loader: string;
    mod_loader_version: string;
    storage_path: string;
    description?: string;
    memory_mb?: number;
}

const Home = () => {
    const { t } = useLocale();
    const [servers, setServers] = useState<Server[]>([]);
    const [serverOrder, setServerOrder] = useState<string[]>([]);
    const [isEditModalOpen, setIsEditModalOpen] = useState(false);
    const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
    const [selectedServer, setSelectedServer] = useState<Server | null>(null);
    const [serverToDelete, setServerToDelete] = useState<Server | null>(null);
    const [currentPage, setCurrentPage] = useState<'home' | 'settings' | 'serverDetails'>('home');
    const [selectedServerForDetails, setSelectedServerForDetails] = useState<Server | null>(null);

    // Cargar servidores y orden al montar el componente
    useEffect(() => {
        initializeApp();
    }, []);

    const initializeApp = async () => {
        try {
            // Primero cargar el orden
            const order = await loadServerOrder();
            console.log('🔄 Loaded order in initializeApp:', order);
            
            // Luego cargar servidores usando el orden cargado
            await loadServersFromJSON(order);
        } catch (error) {
            console.error('Error initializing app:', error);
        }
    };

    const loadServersFromJSON = async (customOrder?: string[]) => {
        try {
            // Clean up any incomplete servers from previous sessions
            console.log('Cleaning up incomplete servers...');
            try {
                const cleanedServers: string[] = await invoke('cleanup_incomplete_servers');
                if (cleanedServers.length > 0) {
                    console.log('Cleaned up incomplete servers:', cleanedServers);
                }
            } catch (error) {
                console.warn('Error cleaning up incomplete servers:', error);
            }
            
            const instances: ServerInstance[] = await invoke('get_all_server_instances');
            
            // Load server data with max players from server.properties and check running status
            const serverList: Server[] = await Promise.all(
                instances.map(async (instance) => {
                    // Fetch max players from server.properties
                    let maxPlayers = 20; // Default value
                    try {
                        maxPlayers = await invoke('get_server_max_players', { 
                            serverName: instance.name 
                        });
                    } catch (error) {
                        console.warn(`Could not load max players for ${instance.name}, using default:`, error);
                    }

                    // Check if server is currently running
                    let isOnline = false;
                    try {
                        isOnline = await invoke('is_server_running', { 
                            serverName: instance.name 
                        });
                    } catch (error) {
                        console.warn(`Could not check running status for ${instance.name}:`, error);
                    }

                    return {
                        id: instance.name, // Usar el nombre como ID único
                        name: instance.name,
                        description: instance.description || `${instance.mod_loader.charAt(0).toUpperCase() + instance.mod_loader.slice(1)} server running Minecraft ${instance.version}`,
                        hasCustomImg: false, // Por ahora sin imagen personalizada
                        imgUrl: '',
                        version: instance.version,
                        serverType: instance.mod_loader,
                        loaderVersion: instance.mod_loader_version,
                        isOnline: isOnline,
                        playerCount: 0, // TODO: Get real player count when server is running
                        maxPlayers: maxPlayers,
                        memory: instance.memory_mb || 2048 // Use configured memory or default 2GB in MB
                    };
                })
            );
            
            // Usar el orden pasado como parámetro o el estado actual
            const currentOrder = customOrder || serverOrder;
            console.log('🎯 Using order for sorting:', currentOrder);
            
            // Aplicar orden personalizado a los servidores cargados
            const orderedServers = sortServersByOrder(serverList, currentOrder);
            setServers(orderedServers);

            // Agregar nuevos servidores al final del orden si no están
            const allServerNames = serverList.map(s => s.name);
            const newServers = allServerNames.filter(name => !currentOrder.includes(name));
            if (newServers.length > 0) {
                const updatedOrder = [...currentOrder, ...newServers];
                console.log('📝 Adding new servers to order:', newServers);
                setServerOrder(updatedOrder);
                saveServerOrder(updatedOrder);
            } else if (customOrder) {
                // Si se pasó un orden personalizado y no hay servidores nuevos, actualizar el estado
                setServerOrder(customOrder);
            }
        } catch (error) {
            console.error('Error loading servers from JSON:', error);
        }
    };

    // Cargar orden de servidores desde el backend
    const loadServerOrder = async (): Promise<string[]> => {
        try {
            const order: string[] = await invoke('get_server_order');
            console.log('📋 Loaded server order:', order);
            setServerOrder(order);
            return order;
        } catch (error) {
            console.warn('Could not load server order, using default:', error);
            setServerOrder([]);
            return [];
        }
    };

    // Guardar orden de servidores al backend
    const saveServerOrder = async (order: string[]) => {
        try {
            await invoke('save_server_order', { order });
            console.log('💾 Server order saved:', order);
        } catch (error) {
            console.error('Error saving server order:', error);
        }
    };

    // Ordenar servidores según el orden personalizado
    const sortServersByOrder = (serverList: Server[], customOrder: string[]): Server[] => {
        return serverList.sort((a, b) => {
            const orderA = customOrder.indexOf(a.name);
            const orderB = customOrder.indexOf(b.name);

            // Si ambos están en el orden personalizado, usar ese orden
            if (orderA !== -1 && orderB !== -1) {
                return orderA - orderB;
            }

            // Si solo uno está en el orden personalizado, ese va primero
            if (orderA !== -1) return -1;
            if (orderB !== -1) return 1;

            // Si ninguno está en el orden personalizado, ordenar alfabéticamente
            return a.name.localeCompare(b.name);
        });
    };

    // Manejar reordenamiento por drag & drop
    const handleReorder = async (newOrder: Server[]) => {
        // Actualizar el estado inmediatamente para UI responsiva
        setServers(newOrder);
        
        // Guardar el nuevo orden al backend
        const newServerOrder = newOrder.map(server => server.name);
        setServerOrder(newServerOrder);
        await saveServerOrder(newServerOrder);
        
        console.log('🔄 Server order updated via drag & drop:', newServerOrder);
    };

    const handleCreateServer = () => {
        // Recargar servidores desde el JSON después de crear uno nuevo, manteniendo el orden actual
        loadServersFromJSON(serverOrder);
    };

    // Removed handleStartStopServer - now handled directly by ServerCard

    const handleEditServer = (serverId: string) => {
        const server = servers.find(s => s.id === serverId);
        if (server) {
            setSelectedServer(server);
            setIsEditModalOpen(true);
        }
    };

    const handleSaveEditedServer = async (updatedServerData: any) => {
        try {
            // Save description to server_config.json
            await invoke('update_server_description', {
                name: updatedServerData.name,
                description: updatedServerData.description || ''
            });

            // Update local state
            setServers(prev => prev.map(server => 
                server.id === updatedServerData.name 
                    ? { ...server, ...updatedServerData, id: updatedServerData.name }
                    : server
            ));
            
            console.log('Server updated:', updatedServerData);
        } catch (error) {
            console.error('Error updating server description:', error);
            alert(`Error saving server: ${error}`);
        }
    };

    const handleCloseEditModal = () => {
        setIsEditModalOpen(false);
        setSelectedServer(null);
    };

    const handleOpenFolder = (serverId: string) => {
        console.log('Open folder for server:', serverId);
        // TODO: Implement file explorer opening
    };

    const handleDeleteServer = (serverId: string) => {
        const server = servers.find(s => s.id === serverId);
        if (server) {
            setServerToDelete(server);
            setIsDeleteModalOpen(true);
        }
    };

    const handleConfirmDelete = async () => {
        if (!serverToDelete) return;
        
        try {
            // Use the new delete command that removes both config and storage folder
            await invoke('delete_server_completely', { name: serverToDelete.name });
            
            // Remove server from order
            const updatedOrder = serverOrder.filter(name => name !== serverToDelete.name);
            setServerOrder(updatedOrder);
            await saveServerOrder(updatedOrder);
            
            // Reload servers from JSON after deletion, usando el orden actualizado
            loadServersFromJSON(updatedOrder);
            console.log('Deleted server:', serverToDelete.name);
        } catch (error) {
            console.error('Error deleting server:', error);
            alert(t('errors.serverDeletionFailed', { error: String(error) }));
        }
    };

    const handleCloseDeleteModal = () => {
        setIsDeleteModalOpen(false);
        setServerToDelete(null);
    };

    const handleOpenSettings = () => {
        setCurrentPage('settings');
    };

    const handleBackToHome = () => {
        setCurrentPage('home');
        setSelectedServerForDetails(null);
    };

    const handleServerClick = (serverId: string) => {
        const server = servers.find(s => s.id === serverId);
        if (server) {
            setSelectedServerForDetails(server);
            setCurrentPage('serverDetails');
        }
    };

    if (currentPage === 'settings') {
        return <Settings onBack={handleBackToHome} />;
    }

    if (currentPage === 'serverDetails' && selectedServerForDetails) {
        return <ServerDetails server={selectedServerForDetails} onBack={handleBackToHome} />;
    }

    return (
        <div className="h-screen pt-8 bg-surface">
            <AllayLayout />
            <ActionBar 
                onCreateServer={handleCreateServer}
                onOpenSettings={handleOpenSettings}
            />

            {servers.length === 0 ? (
                <div className="flex flex-col justify-center items-center h-full gap-4 opacity-30">
                    <img src="/profile-off.png" alt="Allay Off Icon" className="w-30 h-30 drop-shadow-lg drop-shadow-gray-900"/>
                    <p className="text-center text-balance text-text-muted">
                        {t('home.noServers.title')}<br />
                        {t('home.noServers.description')}
                    </p>
                </div>
            ) : (
                <ScrollContainer className="p-4 pt-20" maxHeight="calc(100vh - 80px)">
                    <div className="max-w-4xl mx-auto">
                        <DraggableServerList
                            servers={servers}
                            onEdit={handleEditServer}
                            onOpenFolder={handleOpenFolder}
                            onDelete={handleDeleteServer}
                            onClick={handleServerClick}
                            onReorder={handleReorder}
                        />
                    </div>
                </ScrollContainer>
            )}

            {/* Edit Server Modal */}
            {selectedServer && (
                <EditServerModal
                    isOpen={isEditModalOpen}
                    onClose={handleCloseEditModal}
                    onSaveServer={handleSaveEditedServer}
                    serverData={selectedServer}
                />
            )}

            {/* Delete Server Modal */}
            {serverToDelete && (
                <DeleteServerModal
                    isOpen={isDeleteModalOpen}
                    onClose={handleCloseDeleteModal}
                    onConfirmDelete={handleConfirmDelete}
                    serverName={serverToDelete.name}
                    serverImage={serverToDelete.imgUrl || '/profile.png'}
                />
            )}
        </div>
    );
};

export default Home;