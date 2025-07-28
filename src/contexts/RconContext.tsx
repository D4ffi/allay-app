import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface RconConfig {
    host: string;
    port: number;
    password: string;
}

interface RconConnection {
    serverName: string;
    isConnected: boolean;
    isConnecting: boolean;
    error: string | null;
    config: RconConfig;
}

interface RconContextType {
    connections: Map<string, RconConnection>;
    getConnection: (serverName: string) => RconConnection | undefined;
    connect: (serverName: string, config?: RconConfig) => Promise<void>;
    disconnect: (serverName: string) => Promise<void>;
    executeCommand: (serverName: string, command: string) => Promise<string>;
    isConnected: (serverName: string) => boolean;
    isConnecting: (serverName: string) => boolean;
    getError: (serverName: string) => string | null;
}

const RconContext = createContext<RconContextType | undefined>(undefined);

const DEFAULT_RCON_CONFIG: RconConfig = {
    host: '127.0.0.1',
    port: 25575,
    password: 'minecraft'
};

export const RconProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [connections, setConnections] = useState<Map<string, RconConnection>>(new Map());

    // Get connection for a specific server
    const getConnection = useCallback((serverName: string): RconConnection | undefined => {
        return connections.get(serverName);
    }, [connections]);

    // Check if server is running
    const checkServerRunning = useCallback(async (serverName: string): Promise<boolean> => {
        try {
            const isRunning = await invoke<boolean>('is_server_running', { serverName });
            return isRunning;
        } catch (err) {
            console.error('Failed to check server status:', err);
            return false;
        }
    }, []);

    // Get server RCON password dynamically
    const getServerRconPassword = useCallback(async (serverName: string): Promise<string> => {
        try {
            const password = await invoke<string>('get_server_rcon_password', { serverName });
            return password;
        } catch (err) {
            console.warn('Failed to get server RCON password, using default:', err);
            return DEFAULT_RCON_CONFIG.password;
        }
    }, []);

    // Setup RCON configuration
    const setupRcon = useCallback(async (serverName: string, config: RconConfig) => {
        try {
            console.log(`Setting up RCON for server: ${serverName}`, {
                host: config.host,
                port: config.port,
                password: config.password
            });
            
            await invoke('setup_rcon_for_server', {
                serverName,
                host: config.host,
                port: config.port,
                password: config.password
            });
            console.log(`RCON configured for server: ${serverName}`);
        } catch (err) {
            const errorMsg = `Failed to setup RCON: ${err}`;
            console.error(errorMsg);
            throw err;
        }
    }, []);

    // Update connection state
    const updateConnection = useCallback((serverName: string, updates: Partial<RconConnection>) => {
        setConnections(prev => {
            const newConnections = new Map(prev);
            const existing = newConnections.get(serverName);
            
            if (existing) {
                newConnections.set(serverName, { ...existing, ...updates });
            } else {
                newConnections.set(serverName, {
                    serverName,
                    isConnected: false,
                    isConnecting: false,
                    error: null,
                    config: DEFAULT_RCON_CONFIG,
                    ...updates
                });
            }
            
            return newConnections;
        });
    }, []);

    // Connect to RCON
    const connect = useCallback(async (serverName: string, config: RconConfig = DEFAULT_RCON_CONFIG) => {
        const existing = connections.get(serverName);
        if (existing?.isConnecting) return;

        updateConnection(serverName, { isConnecting: true, error: null });

        try {
            // Get dynamic RCON password
            const password = await getServerRconPassword(serverName);
            const dynamicConfig = { ...config, password };
            
            // Setup RCON configuration with dynamic password
            await setupRcon(serverName, dynamicConfig);
            
            // Try to connect directly (no server running checks)
            console.log(`Attempting direct RCON connection to ${serverName}...`);
            await invoke('connect_rcon', { serverName });
            
            // Verify connection status
            const connected = await invoke<boolean>('is_rcon_connected', { serverName });
            
            updateConnection(serverName, { 
                isConnected: connected, 
                isConnecting: false, 
                error: null,
                config: dynamicConfig
            });
            
            console.log(`Connected to RCON server: ${serverName} with password: ${password}`);
        } catch (err) {
            const errorMsg = `Failed to connect to RCON: ${err}`;
            updateConnection(serverName, { 
                isConnected: false, 
                isConnecting: false, 
                error: errorMsg 
            });
            console.error(errorMsg);
            throw err;
        }
    }, [connections, updateConnection, checkServerRunning, getServerRconPassword, setupRcon]);

    // Disconnect from RCON
    const disconnect = useCallback(async (serverName: string) => {
        try {
            await invoke('disconnect_rcon', { serverName });
            updateConnection(serverName, { 
                isConnected: false, 
                error: null 
            });
            console.log(`Disconnected from RCON server: ${serverName}`);
        } catch (err) {
            const errorMsg = `Failed to disconnect from RCON: ${err}`;
            updateConnection(serverName, { error: errorMsg });
            console.error(errorMsg);
        }
    }, [updateConnection]);

    // Execute RCON command (simplified for ephemeral connections)
    const executeCommand = useCallback(async (serverName: string, command: string): Promise<string> => {
        console.log(`🔧 FRONTEND DEBUG: Executing command '${command}' for server '${serverName}'`);
        
        try {
            updateConnection(serverName, { error: null });
            console.log(`🔧 FRONTEND DEBUG: Calling invoke('execute_rcon_command')...`);
            const response = await invoke<string>('execute_rcon_command', {
                serverName,
                command
            });
            console.log(`🔧 FRONTEND DEBUG: Got response: '${response}'`);
            return response;
        } catch (err) {
            console.log(`🔧 FRONTEND DEBUG: Command failed with error: ${err}`);
            const errorMsg = `Failed to execute RCON command: ${err}`;
            updateConnection(serverName, { error: errorMsg });
            throw err;
        }
    }, [updateConnection]);

    // Helper functions
    const isConnected = useCallback((serverName: string): boolean => {
        return connections.get(serverName)?.isConnected ?? false;
    }, [connections]);

    const isConnecting = useCallback((serverName: string): boolean => {
        return connections.get(serverName)?.isConnecting ?? false;
    }, [connections]);

    const getError = useCallback((serverName: string): string | null => {
        return connections.get(serverName)?.error ?? null;
    }, [connections]);

    // Monitor server status and auto-disconnect when servers stop
    useEffect(() => {
        const interval = setInterval(async () => {
            for (const [serverName, connection] of connections) {
                if (connection.isConnected) {
                    try {
                        const isRunning = await checkServerRunning(serverName);
                        if (!isRunning) {
                            console.log(`Server ${serverName} stopped, disconnecting RCON...`);
                            await disconnect(serverName);
                        } else {
                            // Check if RCON is still connected
                            const stillConnected = await invoke<boolean>('is_rcon_connected', { serverName });
                            if (stillConnected !== connection.isConnected) {
                                updateConnection(serverName, { isConnected: stillConnected });
                            }
                        }
                    } catch (err) {
                        console.error(`Error checking server status for ${serverName}:`, err);
                    }
                }
            }
        }, 5000); // Check every 5 seconds

        return () => clearInterval(interval);
    }, [connections, checkServerRunning, disconnect, updateConnection]);

    const value: RconContextType = {
        connections,
        getConnection,
        connect,
        disconnect,
        executeCommand,
        isConnected,
        isConnecting,
        getError
    };

    return (
        <RconContext.Provider value={value}>
            {children}
        </RconContext.Provider>
    );
};

export const useRconContext = (): RconContextType => {
    const context = useContext(RconContext);
    if (context === undefined) {
        throw new Error('useRconContext must be used within a RconProvider');
    }
    return context;
};