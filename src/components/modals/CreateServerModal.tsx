import { useState } from 'react';
import { Modal } from '../common/Modal';
import { Dropdown } from '../common/Dropdown';
import { RadioGroup } from '../common/RadioGroup';
import { ChangeServerImg } from '../common/ChangeServerImg';

interface CreateServerModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export const CreateServerModal = ({ isOpen, onClose }: CreateServerModalProps) => {
    const [selectedVersion, setSelectedVersion] = useState('');
    const [selectedModLoader, setSelectedModLoader] = useState('');
    const [selectedModLoaderVersion, setSelectedModLoaderVersion] = useState('');
    const [serverImage, setServerImage] = useState<File | null>(null);

    // Opciones de versiones de Minecraft
    const minecraftVersions = [
        { value: '1.21.1', label: 'Minecraft 1.21.1' },
        { value: '1.21', label: 'Minecraft 1.21' },
        { value: '1.20.6', label: 'Minecraft 1.20.6' },
        { value: '1.20.4', label: 'Minecraft 1.20.4' },
        { value: '1.20.1', label: 'Minecraft 1.20.1' },
        { value: '1.19.4', label: 'Minecraft 1.19.4' },
        { value: '1.19.2', label: 'Minecraft 1.19.2' },
        { value: '1.18.2', label: 'Minecraft 1.18.2' },
        { value: '1.17.1', label: 'Minecraft 1.17.1' },
        { value: '1.16.5', label: 'Minecraft 1.16.5' }
    ];

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

    // Versiones de mod loaders
    const modLoaderVersions = {
        fabric: [
            { value: '0.16.5', label: 'Fabric Loader 0.16.5' },
            { value: '0.16.4', label: 'Fabric Loader 0.16.4' },
            { value: '0.16.3', label: 'Fabric Loader 0.16.3' },
            { value: '0.16.2', label: 'Fabric Loader 0.16.2' },
            { value: '0.15.11', label: 'Fabric Loader 0.15.11' }
        ],
        forge: [
            { value: '51.0.33', label: 'Forge 51.0.33 (MC 1.21.1)' },
            { value: '50.1.0', label: 'Forge 50.1.0 (MC 1.21)' },
            { value: '47.3.0', label: 'Forge 47.3.0 (MC 1.20.1)' },
            { value: '43.3.13', label: 'Forge 43.3.13 (MC 1.19.2)' },
            { value: '40.2.21', label: 'Forge 40.2.21 (MC 1.18.2)' }
        ],
        neoforge: [
            { value: '21.1.57', label: 'NeoForge 21.1.57 (MC 1.21.1)' },
            { value: '21.0.167', label: 'NeoForge 21.0.167 (MC 1.21)' },
            { value: '20.4.240', label: 'NeoForge 20.4.240 (MC 1.20.4)' },
            { value: '20.2.88', label: 'NeoForge 20.2.88 (MC 1.20.2)' },
            { value: '20.1.10', label: 'NeoForge 20.1.10 (MC 1.20.1)' }
        ],
        paper: [
            { value: '1.21.1-128', label: 'Paper 1.21.1-128 (MC 1.21.1)' },
            { value: '1.21-119', label: 'Paper 1.21-119 (MC 1.21)' },
            { value: '1.20.6-147', label: 'Paper 1.20.6-147 (MC 1.20.6)' },
            { value: '1.20.4-497', label: 'Paper 1.20.4-497 (MC 1.20.4)' },
            { value: '1.20.1-196', label: 'Paper 1.20.1-196 (MC 1.20.1)' },
            { value: '1.19.4-550', label: 'Paper 1.19.4-550 (MC 1.19.4)' },
            { value: '1.19.2-307', label: 'Paper 1.19.2-307 (MC 1.19.2)' },
            { value: '1.18.2-388', label: 'Paper 1.18.2-388 (MC 1.18.2)' },
            { value: '1.17.1-411', label: 'Paper 1.17.1-411 (MC 1.17.1)' },
            { value: '1.16.5-794', label: 'Paper 1.16.5-794 (MC 1.16.5)' }
        ],
        quilt: [
            { value: '0.26.4', label: 'Quilt Loader 0.26.4' },
            { value: '0.26.3', label: 'Quilt Loader 0.26.3' },
            { value: '0.26.0', label: 'Quilt Loader 0.26.0' },
            { value: '0.25.1', label: 'Quilt Loader 0.25.1' },
            { value: '0.24.0', label: 'Quilt Loader 0.24.0' }
        ]
    };

    const handleModLoaderChange = (value: string) => {
        setSelectedModLoader(value);
        // Limpiar la versión del mod loader cuando se cambia a vanilla o se cambia de tipo
        if (value === 'vanilla' || value !== selectedModLoader) {
            setSelectedModLoaderVersion('');
        }
    };

    const handleMinecraftVersionChange = (value: string) => {
        setSelectedVersion(value);
        // Limpiar la versión del mod loader cuando se cambia la versión de Minecraft
        setSelectedModLoaderVersion('');
    };

    // Obtener las versiones disponibles para el mod loader seleccionado
    const getAvailableModLoaderVersions = () => {
        if (selectedModLoader === 'vanilla' || !selectedModLoader) {
            return [];
        }
        return modLoaderVersions[selectedModLoader as keyof typeof modLoaderVersions] || [];
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
    };

    // Función para cerrar el modal y resetear opciones
    const closeModal = () => {
        onClose();
        resetModalOptions();
    };

    const handleCreateServer = () => {
        // Aquí iría la lógica para crear el servidor
        console.log('Creating server:', {
            version: selectedVersion,
            modLoader: selectedModLoader,
            modLoaderVersion: selectedModLoaderVersion,
            serverImage: serverImage
        });
        closeModal();
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
                        className="border border-gray-300 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent w-full"
                    />
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                        Minecraft Version
                    </label>
                    <Dropdown
                        options={minecraftVersions}
                        placeholder="Select Minecraft version..."
                        value={selectedVersion}
                        onChange={handleMinecraftVersionChange}
                        className="w-full"
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
                            placeholder={`Select ${selectedModLoader} version...`}
                            value={selectedModLoaderVersion}
                            onChange={setSelectedModLoaderVersion}
                            className="w-full"
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