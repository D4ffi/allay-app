import { useState, useEffect } from 'react';
import { Modal } from '../common/Modal';
import { RadioGroup } from '../common/RadioGroup';
import { ChangeServerImg } from '../common/ChangeServerImg';
import { MemorySlider } from '../common/MemorySlider';
import { invoke } from '@tauri-apps/api/core';
import { MinecraftMOTD } from '../common/MinecraftMOTD';

interface EditServerModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSaveServer: (serverData: any) => void;
    serverData: {
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
    };
}

export const EditServerModal = ({ isOpen, onClose, onSaveServer, serverData }: EditServerModalProps) => {
    const [serverName, setServerName] = useState('');
    const [serverImage, setServerImage] = useState<File | null>(null);
    const [serverImageUrl, setServerImageUrl] = useState('');
    const [descriptionType, setDescriptionType] = useState('custom');
    const [customDescription, setCustomDescription] = useState('');
    const [motdValue, setMotdValue] = useState('');
    const [memoryAllocation, setMemoryAllocation] = useState(2048); // Default 2GB in MB
    
    // Loading states
    const [isLoadingMotd, setIsLoadingMotd] = useState(false);
    const [motdError, setMotdError] = useState<string | null>(null);
    const [isSaving, setIsSaving] = useState(false);
    const [saveError, setSaveError] = useState<string | null>(null);

    // Load server data when modal opens
    useEffect(() => {
        if (isOpen && serverData) {
            setServerName(serverData.name);
            setCustomDescription(serverData.description);
            setServerImageUrl(serverData.imgUrl);
            setMemoryAllocation(serverData.memory || 2048); // Default 2GB in MB
            
            // Determine if description is custom or MOTD-based
            // We'll assume it's custom for now, but we could add logic to detect MOTD
            setDescriptionType('custom');
            
            // Load MOTD from server.properties
            loadServerMotd();
        }
    }, [isOpen, serverData]);

    const loadServerMotd = async () => {
        if (!serverData?.name) return;
        
        setIsLoadingMotd(true);
        setMotdError(null);
        
        try {
            const motd: string = await invoke('get_server_motd', {
                serverName: serverData.name
            });
            setMotdValue(motd);
        } catch (error) {
            console.error('Error loading MOTD:', error);
            setMotdError(`Failed to load MOTD: ${error}`);
            setMotdValue('A Minecraft Server');
        } finally {
            setIsLoadingMotd(false);
        }
    };

    const handleServerImageChange = (file: File | null, imageUrl: string) => {
        setServerImage(file);
        if (file) {
            setServerImageUrl(imageUrl);
        }
    };

    const descriptionOptions = [
        {
            value: 'custom',
            label: 'Custom Description',
            description: 'Enter a custom description for your server'
        },
        {
            value: 'motd',
            label: 'Use MOTD',
            description: 'Use the MOTD from server.properties file'
        }
    ];


    const resetModalData = () => {
        setServerName('');
        setServerImage(null);
        setServerImageUrl('');
        setDescriptionType('custom');
        setCustomDescription('');
        setMotdValue('');
        setMemoryAllocation(2048);
        setMotdError(null);
        setSaveError(null);
        setIsSaving(false);
    };

    const closeModal = () => {
        onClose();
        resetModalData();
    };

    const handleSaveServer = async () => {
        // Clear previous errors
        setSaveError(null);
        
        // Validate required fields
        if (!serverName.trim()) {
            setSaveError('Please enter a server name');
            return;
        }
        
        if (descriptionType === 'custom' && !customDescription.trim()) {
            setSaveError('Please enter a custom description');
            return;
        }
        
        setIsSaving(true);
        
        try {
            // Determine final description based on type
            const finalDescription = descriptionType === 'motd' ? motdValue : customDescription;
            
            // Create updated server data
            const updatedServer = {
                ...serverData,
                name: serverName.trim(),
                description: finalDescription,
                hasCustomImg: serverImage !== null || serverData.hasCustomImg,
                imgUrl: serverImageUrl || serverData.imgUrl,
                memory: memoryAllocation
            };
            
            // TODO: Call backend to update server properties/config
            // await invoke('update_server_config', { serverData: updatedServer });
            
            // Call parent function to update the server
            onSaveServer(updatedServer);
            
            closeModal();
            
        } catch (error) {
            console.error('Error saving server:', error);
            setSaveError(`Failed to save server: ${error}`);
        } finally {
            setIsSaving(false);
        }
    };

    if (!serverData) return null;

    return (
        <Modal
            isOpen={isOpen}
            onClose={closeModal}
            title={`Edit Server - ${serverData.name}`}
            size="lg"
        >
            <div className="relative">
                {/* Progress/Error Section */}
                {(isSaving || saveError) && (
                    <div className="sticky top-0 z-10 bg-white border-b border-gray-200 pb-4 mb-6">
                        <div className="bg-gray-50 rounded-lg p-4 border shadow-sm">
                            {isSaving && (
                                <div className="flex items-center space-x-3">
                                    <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
                                    <div className="text-sm font-medium text-gray-700">
                                        Saving server changes...
                                    </div>
                                </div>
                            )}
                            {saveError && (
                                <div className="flex items-center space-x-3 text-red-700">
                                    <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                                    </svg>
                                    <div className="text-sm font-medium">{saveError}</div>
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
                                initialImageUrl={serverData.imgUrl}
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
                            disabled={isSaving}
                            className={`w-full border-2 rounded-lg px-4 py-3 text-gray-900 placeholder-gray-500 transition-all duration-200 ${
                                isSaving 
                                    ? 'border-gray-200 bg-gray-100 cursor-not-allowed opacity-60' 
                                    : 'border-gray-200 bg-white hover:border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500'
                            }`}
                        />
                    </div>

                    {/* Memory Allocation Section */}
                    <div>
                        <MemorySlider
                            value={memoryAllocation}
                            onChange={setMemoryAllocation}
                            disabled={isSaving}
                        />
                    </div>

                    {/* Description Type Section */}
                    <div>
                        <label className="block text-sm font-semibold text-gray-800 mb-4">
                            Server Description
                        </label>
                        <div className="bg-gray-50 rounded-xl p-4">
                            <RadioGroup
                                name="descriptionType"
                                options={descriptionOptions}
                                value={descriptionType}
                                onChange={setDescriptionType}
                                layout="vertical"
                                disabled={isSaving}
                            />
                        </div>
                    </div>

                    {/* Custom Description Input */}
                    {descriptionType === 'custom' && (
                        <div className="transition-all duration-300 ease-in-out animate-in slide-in-from-top-2">
                            <label className="block text-sm font-semibold text-gray-800 mb-3">
                                Custom Description
                            </label>
                            <textarea
                                placeholder="Enter a description for your server..."
                                value={customDescription}
                                onChange={(e) => setCustomDescription(e.target.value)}
                                disabled={isSaving}
                                rows={3}
                                className={`w-full border-2 rounded-lg px-4 py-3 text-gray-900 placeholder-gray-500 transition-all duration-200 resize-none ${
                                    isSaving 
                                        ? 'border-gray-200 bg-gray-100 cursor-not-allowed opacity-60' 
                                        : 'border-gray-200 bg-white hover:border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500'
                                }`}
                            />
                        </div>
                    )}

                    {/* MOTD Preview */}
                    {descriptionType === 'motd' && (
                        <div className="transition-all duration-300 ease-in-out animate-in slide-in-from-top-2">
                            <label className="block text-sm font-semibold text-gray-800 mb-3">
                                MOTD Preview
                            </label>
                            <div className="bg-gray-900 rounded-lg p-4 border-2 border-gray-300">
                                {isLoadingMotd ? (
                                    <div className="flex items-center space-x-3">
                                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                                        <span className="text-white text-sm">Loading MOTD...</span>
                                    </div>
                                ) : motdError ? (
                                    <div className="text-red-400 text-sm">{motdError}</div>
                                ) : (
                                    <MinecraftMOTD motd={motdValue} />
                                )}
                            </div>
                        </div>
                    )}

                    {/* Server Info Display */}
                    <div className="bg-blue-50 rounded-lg p-4 border border-blue-200">
                        <h3 className="text-sm font-semibold text-blue-800 mb-2">Server Information</h3>
                        <div className="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <span className="text-blue-600 font-medium">Version:</span>
                                <span className="text-blue-700 ml-2">{serverData.version}</span>
                            </div>
                            <div>
                                <span className="text-blue-600 font-medium">Type:</span>
                                <span className="text-blue-700 ml-2 capitalize">{serverData.serverType}</span>
                            </div>
                            {serverData.loaderVersion && (
                                <div>
                                    <span className="text-blue-600 font-medium">Loader Version:</span>
                                    <span className="text-blue-700 ml-2">{serverData.loaderVersion}</span>
                                </div>
                            )}
                            <div>
                                <span className="text-blue-600 font-medium">Max Players:</span>
                                <span className="text-blue-700 ml-2">{serverData.maxPlayers}</span>
                            </div>
                        </div>
                    </div>

                    {/* Action Buttons */}
                    <div className="flex justify-end space-x-3 pt-6 border-t-2 border-gray-100">
                        <button
                            onClick={closeModal}
                            disabled={isSaving}
                            className="px-6 py-3 text-sm font-semibold text-gray-700 bg-white border-2 border-gray-300 rounded-lg hover:bg-gray-50 hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-all duration-200 shadow-sm disabled:opacity-60 disabled:cursor-not-allowed"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={handleSaveServer}
                            disabled={isSaving}
                            className="px-6 py-3 text-sm font-semibold text-white bg-gradient-to-r from-blue-600 to-blue-700 border border-transparent rounded-lg hover:from-blue-700 hover:to-blue-800 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all duration-200 shadow-lg hover:shadow-xl disabled:opacity-60 disabled:cursor-not-allowed"
                        >
                            {isSaving ? 'Saving...' : 'Save Changes'}
                        </button>
                    </div>
                </div>
            </div>
        </Modal>
    );
};