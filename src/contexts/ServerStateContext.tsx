import React, { createContext, useContext, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export type ServerStatus = 'offline' | 'online';

interface ServerStateContextType {
    getServerStatus: (serverName: string) => ServerStatus;
    setServerStatus: (serverName: string, status: ServerStatus) => void;
    startServer: (serverName: string) => Promise<void>;
    stopServer: (serverName: string, b: boolean) => Promise<void>;
}

const ServerStateContext = createContext<ServerStateContextType | undefined>(undefined);

export const ServerStateProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [servers, setServers] = useState<Record<string, ServerStatus>>({});

    const getServerStatus = useCallback((serverName: string): ServerStatus => {
        return servers[serverName] ?? 'offline';
    }, [servers]);

    const setServerStatus = useCallback((serverName: string, status: ServerStatus) => {
        setServers(prev => ({
            ...prev,
            [serverName]: status
        }));
    }, []);

    const startServer = useCallback(async (serverName: string) => {
        try {
            // 1. Cambiar UI inmediatamente a online
            setServerStatus(serverName, 'online');
            
            // 2. Obtener tipo de loader
            const loaderType = await invoke<string>('get_server_loader_type', {
                serverName
            });

            // 3. Iniciar servidor en backend
            await invoke<string>('start_server', {
                serverName,
                loader: loaderType
            });
            
            console.log(`✅ Server ${serverName} started successfully`);
        } catch (error) {
            console.error(`❌ Failed to start server ${serverName}:`, error);
            // Si falla, volver a offline
            setServerStatus(serverName, 'offline');
            throw error;
        }
    }, [setServerStatus]);

    const stopServer = useCallback(async (serverName: string) => {
        try {
            // 1. Cambiar UI inmediatamente a offline
            setServerStatus(serverName, 'offline');
            
            // 2. Parar servidor en backend
            await invoke<string>('stop_server', { serverName });
            
            console.log(`✅ Server ${serverName} stopped successfully`);
        } catch (error) {
            console.error(`❌ Failed to stop server ${serverName}:`, error);
            throw error;
        }
    }, [setServerStatus]);

    const value: ServerStateContextType = {
        getServerStatus,
        setServerStatus,
        startServer,
        stopServer
    };

    return (
        <ServerStateContext.Provider value={value}>
            {children}
        </ServerStateContext.Provider>
    );
};

export const useServerState = (): ServerStateContextType => {
    const context = useContext(ServerStateContext);
    if (context === undefined) {
        throw new Error('useServerState must be used within a ServerStateProvider');
    }
    return context;
};