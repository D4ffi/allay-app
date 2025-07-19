import { useState, useEffect } from 'react';
import { Modal } from '../common/Modal';
import { Dropdown } from '../common/Dropdown';
import { RadioGroup } from '../common/RadioGroup';
import { ChangeServerImg } from '../common/ChangeServerImg';
import { invoke } from '@tauri-apps/api/core';

interface CreateServerModalProps {
    isOpen: boolean;
    onClose: () => void;
    onCreateServer: (serverData: any) => void;
}

interface MinecraftVersion {
    id: string;
    version_type: string;
    loader: string;
    release_time: string;
    latest: boolean;
    recommended: boolean;
    minecraft_version?: string;
}

interface VersionResponse {
    latest?: MinecraftVersion;
    recommended?: MinecraftVersion;
    versions: MinecraftVersion[];
}

export const CreateServerModal = ({ isOpen, onClose, onCreateServer }: CreateServerModalProps) => {
    const [selectedVersion, setSelectedVersion] = useState('');
    const [selectedModLoader, setSelectedModLoader] = useState('');
    const [selectedModLoaderVersion, setSelectedModLoaderVersion] = useState('');
    const [serverImage, setServerImage] = useState<File | null>(null);
    const [serverName, setServerName] = useState('');
    
    // API data states
    const [vanillaVersions, setVanillaVersions] = useState<MinecraftVersion[]>([]);
    const [loaderVersions, setLoaderVersions] = useState<Record<string, MinecraftVersion[]>>({});
    const [isLoadingVersions, setIsLoadingVersions] = useState(false);
    const [versionsError, setVersionsError] = useState<string | null>(null);

    // Load versions from API on modal open
    useEffect(() => {
        if (isOpen) {
            loadMinecraftVersions();
        }
    }, [isOpen]);

    const loadMinecraftVersions = async () => {
        setIsLoadingVersions(true);
        setVersionsError(null);
        
        try {
            // Clear vanilla cache first to ensure we get all versions
            await invoke('clear_version_cache', { loader: 'vanilla' });
            
            // Load vanilla versions for Minecraft version dropdown (force refresh to get all versions)
            const vanillaResponse: VersionResponse = await invoke('get_minecraft_versions', {
                loader: 'vanilla',
                forceRefresh: true
            });
            setVanillaVersions(vanillaResponse.versions);
            
            console.log(`Loaded ${vanillaResponse.versions.length} vanilla versions`);
        } catch (error) {
            console.error('Error loading vanilla versions:', error);
            setVersionsError('Failed to load Minecraft versions');
        } finally {
            setIsLoadingVersions(false);
        }
    };

    const loadLoaderVersions = async (loader: string, mcVersion?: string) => {
        if (loader === 'vanilla') return;
        
        try {
            const response: VersionResponse = await invoke('get_minecraft_versions', {
                loader: loader,
                forceRefresh: false,
                minecraftVersion: mcVersion || null
            });
            
            // Create a cache key that includes MC version for specific requests
            const cacheKey = mcVersion ? `${loader}-${mcVersion}` : loader;
            
            setLoaderVersions(prev => ({
                ...prev,
                [cacheKey]: response.versions
            }));
            
            console.log(`Loaded ${response.versions.length} ${loader} versions for MC ${mcVersion || 'all'}`);
        } catch (error) {
            console.error(`Error loading ${loader} versions:`, error);
        }
    };

    // Get formatted Minecraft versions for dropdown
    const getMinecraftVersionOptions = () => {
        if (vanillaVersions.length === 0) {
            return [{ value: '', label: 'Loading versions...' }];
        }
        
        return vanillaVersions.map(version => ({
            value: version.id,
            label: `Minecraft ${version.id}${version.latest ? ' (Latest)' : ''}${version.recommended ? ' (Recommended)' : ''}`
        }));
    };

    // Opciones de mod loaders
    const modLoaders = [
        { 
            value: 'vanilla', 
            label: 'Vanilla', 
            description: 'Pure Minecraft without mods' 
        },
        { 
            value: 'fabric', 
            label: 'Fabric', 
            description: 'Lightweight and modern modding platform' 
        },
        { 
            value: 'forge', 
            label: 'Forge', 
            description: 'The most popular modding platform' 
        },
        { 
            value: 'neoforge', 
            label: 'NeoForge', 
            description: 'Modern fork of Forge with improvements' 
        },
        { 
            value: 'paper', 
            label: 'Paper', 
            description: 'High-performance Minecraft server software' 
        },
        { 
            value: 'quilt', 
            label: 'Quilt', 
            description: 'Community-driven fork of Fabric' 
        }
    ];

    const handleModLoaderChange = (value: string) => {
        setSelectedModLoader(value);
        // Limpiar la versión del mod loader cuando se cambia a vanilla o se cambia de tipo
        if (value === 'vanilla' || value !== selectedModLoader) {
            setSelectedModLoaderVersion('');
        }
        
        // Load versions for the selected loader with the current MC version
        if (value !== 'vanilla' && selectedVersion) {
            loadLoaderVersions(value, selectedVersion);
        }
    };

    const handleMinecraftVersionChange = (value: string) => {
        setSelectedVersion(value);
        // Limpiar la versión del mod loader cuando se cambia la versión de Minecraft
        setSelectedModLoaderVersion('');
        
        // Reload loader versions if a loader is already selected
        if (selectedModLoader && selectedModLoader !== 'vanilla' && value) {
            loadLoaderVersions(selectedModLoader, value);
        }
    };

    // Obtener las versiones disponibles para el mod loader seleccionado
    const getAvailableModLoaderVersions = () => {
        if (selectedModLoader === 'vanilla' || !selectedModLoader) {
            return [];
        }
        
        // Use cache key that includes MC version if available
        const cacheKey = selectedVersion ? `${selectedModLoader}-${selectedVersion}` : selectedModLoader;
        const versions = loaderVersions[cacheKey];
        
        if (!versions) {
            return [{ value: '', label: 'Loading versions...' }];
        }
        
        return versions.map(version => {
            let label = version.id;
            if (version.minecraft_version) {
                label += ` (MC ${version.minecraft_version})`;
            }
            if (version.latest) label += ' (Latest)';
            if (version.recommended) label += ' (Recommended)';
            
            return {
                value: version.id,
                label: label
            };
        });
    };


    const handleServerImageChange = (file: File | null, imageUrl: string) => {
        setServerImage(file);
        console.log('Server image changed:', file ? file.name : 'default', imageUrl);
    };

    // Función para resetear todas las opciones del modal
    const resetModalOptions = () => {
        setSelectedVersion('');
        setSelectedModLoader('');
        setSelectedModLoaderVersion('');
        setServerImage(null);
        setServerName('');
        setVersionsError(null);
    };

    // Función para cerrar el modal y resetear opciones
    const closeModal = () => {
        onClose();
        resetModalOptions();
    };

    const handleCreateServer = async () => {
        // Validar que todos los campos requeridos estén llenos
        if (!serverName.trim()) {
            alert('Please enter a server name');
            return;
        }
        
        if (!selectedVersion) {
            alert('Please select a Minecraft version');
            return;
        }
        
        if (!selectedModLoader) {
            alert('Please select a mod loader');
            return;
        }
        
        if (selectedModLoader !== 'vanilla' && !selectedModLoaderVersion) {
            alert('Please select a mod loader version');
            return;
        }
        
        try {
            // Crear la instancia del servidor en el backend
            const result = await invoke('create_server_instance', {
                name: serverName.trim(),
                version: selectedVersion,
                modLoader: selectedModLoader,
                modLoaderVersion: selectedModLoaderVersion || 'none'
            });
            
            console.log('Server created:', result);
            
            // Crear el objeto servidor para el frontend
            const newServer = {
                name: serverName.trim(),
                description: `${selectedModLoader.charAt(0).toUpperCase() + selectedModLoader.slice(1)} server running Minecraft ${selectedVersion}`,
                hasCustomImg: serverImage !== null,
                imgUrl: serverImage ? URL.createObjectURL(serverImage) : '',
                version: selectedVersion,
                serverType: selectedModLoader,
                loaderVersion: selectedModLoaderVersion || '',
                isOnline: false,
                playerCount: 0,
                maxPlayers: 20
            };
            
            // Llamar a la función del padre para agregar el servidor
            onCreateServer(newServer);
            
            closeModal();
        } catch (error) {
            console.error('Error creating server:', error);
            alert(`Error creating server: ${error}`);
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={closeModal}
            title="Create Server"
            size="md"
        >
            <div className="space-y-4">
                {/* Server Image */}
                <div className="flex justify-center">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-3 text-center">
                            Server Icon
                        </label>
                        <ChangeServerImg
                            size="lg"
                            onImageChange={handleServerImageChange}
                            className="mx-auto"
                        />
                    </div>
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                        Server Name
                    </label>
                    <input
                        type="text"
                        placeholder="My awesome server"
                        value={serverName}
                        onChange={(e) => setServerName(e.target.value)}
                        className="border border-gray-300 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent w-full"
                    />
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                        Minecraft Version
                    </label>
                    {versionsError && (
                        <div className="mb-2 p-2 text-sm text-red-600 bg-red-50 border border-red-200 rounded">
                            {versionsError}
                        </div>
                    )}
                    <Dropdown
                        options={getMinecraftVersionOptions()}
                        placeholder={isLoadingVersions ? "Loading versions..." : "Select Minecraft version..."}
                        value={selectedVersion}
                        onChange={handleMinecraftVersionChange}
                        className="w-full"
                        disabled={isLoadingVersions}
                    />
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-3">
                        Mod Loader
                    </label>
                    <RadioGroup
                        name="modLoader"
                        options={modLoaders}
                        value={selectedModLoader}
                        onChange={handleModLoaderChange}
                        layout="grid"
                    />
                </div>

                {/* Dropdown condicional para versiones de mod loader */}
                {selectedVersion && selectedModLoader && selectedModLoader !== 'vanilla' && (
                    <div className="transition-all duration-200 ease-in-out">
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            {selectedModLoader.charAt(0).toUpperCase() + selectedModLoader.slice(1)} Version
                        </label>
                        <Dropdown
                            options={getAvailableModLoaderVersions()}
                            placeholder={(() => {
                                const cacheKey = selectedVersion ? `${selectedModLoader}-${selectedVersion}` : selectedModLoader;
                                return loaderVersions[cacheKey] ? `Select ${selectedModLoader} version...` : "Loading versions...";
                            })()}
                            value={selectedModLoaderVersion}
                            onChange={setSelectedModLoaderVersion}
                            className="w-full"
                            disabled={(() => {
                                const cacheKey = selectedVersion ? `${selectedModLoader}-${selectedVersion}` : selectedModLoader;
                                return !loaderVersions[cacheKey];
                            })()}
                        />
                    </div>
                )}

                {/* Botones Done y Cancel */}
                <div className="flex justify-end space-x-3 pt-4 border-t border-gray-200">
                    <button
                        onClick={closeModal}
                        className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 cursor-pointer focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-colors duration-200"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={handleCreateServer}
                        className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 cursor-pointer focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors duration-200"
                    >
                        Done
                    </button>
                </div>
            </div>
        </Modal>
    );
};