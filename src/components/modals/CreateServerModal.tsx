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
    
    // Server creation states
    const [isCreatingServer, setIsCreatingServer] = useState(false);
    const [creationProgress, setCreationProgress] = useState('');
    const [creationError, setCreationError] = useState<string | null>(null);

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
            // Clear the vanilla cache first to ensure we get all versions
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
            
            // Create a cache key that includes an MC version for specific requests
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
        // Limpiar la selección del mod loader y su versión cuando se cambia o limpia la versión de Minecraft
        if (!value) {
            setSelectedModLoader('');
        }
        setSelectedModLoaderVersion('');
        
        // Reload loader versions if a loader is already selected and there's a value
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
        setCreationError(null);
        setCreationProgress('');
        setIsCreatingServer(false);
    };

    // Función para cerrar el modal y resetear opciones
    const closeModal = () => {
        onClose();
        resetModalOptions();
    };

    const handleCreateServer = async () => {
        // Clear previous errors
        setCreationError(null);
        
        // Validar que todos los campos requeridos estén llenos
        if (!serverName.trim()) {
            setCreationError('Please enter a server name');
            return;
        }
        
        if (!selectedVersion) {
            setCreationError('Please select a Minecraft version');
            return;
        }
        
        if (!selectedModLoader) {
            setCreationError('Please select a mod loader');
            return;
        }
        
        if (selectedModLoader !== 'vanilla' && !selectedModLoaderVersion) {
            setCreationError('Please select a mod loader version');
            return;
        }
        
        setIsCreatingServer(true);
        
        try {
            // Step 1: Create server instance structure
            setCreationProgress('Creating server instance...');
            const createResult = await invoke('create_server_instance', {
                name: serverName.trim(),
                version: selectedVersion,
                modLoader: selectedModLoader,
                modLoaderVersion: selectedModLoaderVersion || 'none'
            });
            
            console.log('Server instance created:', createResult);
            
            // Step 2: Download server JAR
            setCreationProgress('Downloading server JAR file...');
            const downloadResult = await invoke('download_server_jar', {
                serverName: serverName.trim(),
                loader: selectedModLoader,
                minecraftVersion: selectedVersion,
                loaderVersion: selectedModLoader !== 'vanilla' ? selectedModLoaderVersion : null
            });
            
            console.log('Server JAR downloaded:', downloadResult);
            
            // Step 3: Setup server (install, generate files, etc.)
            setCreationProgress('Setting up server environment...');
            const setupResult = await invoke('setup_server', {
                serverName: serverName.trim(),
                loader: selectedModLoader,
                minecraftVersion: selectedVersion,
                loaderVersion: selectedModLoader !== 'vanilla' ? selectedModLoaderVersion : null
            });
            
            console.log('Server setup completed:', setupResult);
            
            // Step 4: Complete server setup
            setCreationProgress('Finalizing server configuration...');
            
            // Create the server object for the frontend
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
            
            setCreationProgress('Server created successfully!');
            
            // Call parent function to add the server
            onCreateServer(newServer);
            
            // Small delay to show success message
            setTimeout(() => {
                closeModal();
            }, 1000);
            
        } catch (error) {
            console.error('Error creating server:', error);
            setCreationError(`Failed to create server: ${error}`);
        } finally {
            setIsCreatingServer(false);
            setCreationProgress('');
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={closeModal}
            title="Create New Server"
            size="lg"
        >
            <div className="relative">
                {/* Fixed Progress Section */}
                {(isCreatingServer || creationProgress || creationError) && (
                    <div className="sticky top-0 z-10 bg-white border-b border-gray-200 pb-4 mb-6">
                        <div className="bg-gray-50 rounded-lg p-4 border shadow-sm">
                            {isCreatingServer && (
                                <div className="flex items-center space-x-3">
                                    <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
                                    <div className="text-sm font-medium text-gray-700">
                                        {creationProgress || 'Creating server...'}
                                    </div>
                                </div>
                            )}
                            {creationError && (
                                <div className="flex items-center space-x-3 text-red-700">
                                    <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                                    </svg>
                                    <div className="text-sm font-medium">{creationError}</div>
                                </div>
                            )}
                            {creationProgress && !isCreatingServer && !creationError && (
                                <div className="flex items-center space-x-3 text-green-700">
                                    <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                                    </svg>
                                    <div className="text-sm font-medium">{creationProgress}</div>
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {/* Scrollable Content */}
                <div className="space-y-6">
                    {/* Server Image Section */}
                <div className="flex justify-center bg-gradient-to-br from-blue-50 to-indigo-50 rounded-xl p-6">
                    <div>
                        <label className="block text-sm font-semibold text-gray-800 mb-4 text-center">
                            Server Icon
                        </label>
                        <ChangeServerImg
                            size="lg"
                            onImageChange={handleServerImageChange}
                            className="mx-auto shadow-lg"
                        />
                    </div>
                </div>

                {/* Server Name Section */}
                <div>
                    <label className="block text-sm font-semibold text-gray-800 mb-3">
                        Server Name
                    </label>
                    <input
                        type="text"
                        placeholder="Enter your server name..."
                        value={serverName}
                        onChange={(e) => setServerName(e.target.value)}
                        disabled={isCreatingServer}
                        className={`w-full border-2 rounded-lg px-4 py-3 text-gray-900 placeholder-gray-500 transition-all duration-200 ${
                            isCreatingServer 
                                ? 'border-gray-200 bg-gray-100 cursor-not-allowed opacity-60' 
                                : 'border-gray-200 bg-white hover:border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500'
                        }`}
                    />
                </div>

                {/* Minecraft Version Section */}
                <div>
                    <label className="block text-sm font-semibold text-gray-800 mb-3">
                        Minecraft Version
                    </label>
                    {versionsError && (
                        <div className="mb-3 p-3 text-sm text-red-700 bg-red-50 border-l-4 border-red-400 rounded-r-lg">
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

                {/* Mod Loader Section */}
                <div>
                    <label className="block text-sm font-semibold text-gray-800 mb-4">
                        Mod Loader
                    </label>
                    <div className="bg-gray-50 rounded-xl p-4">
                        <RadioGroup
                            name="modLoader"
                            options={modLoaders}
                            value={selectedModLoader}
                            onChange={handleModLoaderChange}
                            layout="grid"
                            disabled={!selectedVersion}
                        />
                    </div>
                </div>

                {/* Loader Version Section */}
                {selectedVersion && selectedModLoader && selectedModLoader !== 'vanilla' && (
                    <div className="transition-all duration-300 ease-in-out animate-in slide-in-from-top-2">
                        <label className="block text-sm font-semibold text-gray-800 mb-3">
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

                    {/* Action Buttons */}
                    <div className="flex justify-end space-x-3 pt-6 border-t-2 border-gray-100">
                        <button
                            onClick={closeModal}
                            className="px-6 py-3 text-sm font-semibold text-gray-700 bg-white border-2 border-gray-300 rounded-lg hover:bg-gray-50 hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-all duration-200 shadow-sm"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={handleCreateServer}
                            className="px-6 py-3 text-sm font-semibold text-white bg-gradient-to-r from-blue-600 to-blue-700 border border-transparent rounded-lg hover:from-blue-700 hover:to-blue-800 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all duration-200 shadow-lg hover:shadow-xl"
                        >
                            Create Server
                        </button>
                    </div>
                </div>
            </div>
        </Modal>
    );
};