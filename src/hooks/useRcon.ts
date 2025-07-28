import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface RconConfig {
    host: string;
    port: number;
    password: string;
}

export interface UseRconProps {
    serverName: string;
    autoConnect?: boolean;
    rconConfig?: RconConfig;
}

export interface UseRconReturn {
    isConnected: boolean;
    isConnecting: boolean;
    error: string | null;
    connect: () => Promise<void>;
    disconnect: () => Promise<void>;
    executeCommand: (command: string) => Promise<string>;
    setupRcon: (config: RconConfig) => Promise<void>;
    testConnection: () => Promise<boolean>;
}

const DEFAULT_RCON_CONFIG: RconConfig = {
    host: '127.0.0.1',
    port: 25575,
    password: 'minecraft'
};

export const useRcon = ({ 
    serverName, 
    autoConnect = false, 
    rconConfig = DEFAULT_RCON_CONFIG 
}: UseRconProps): UseRconReturn => {
    const [isConnected, setIsConnected] = useState(false);
    const [isConnecting, setIsConnecting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Check connection status (simplified)
    const checkConnectionStatus = useCallback(async () => {
        try {
            const connected = await invoke<boolean>('is_rcon_connected', {
                serverName
            });
            
            // If state changed, update immediately
            if (connected !== isConnected) {
                console.log(`RCON status changed for ${serverName}: ${isConnected} -> ${connected}`);
                setIsConnected(connected);
            }
            
            return connected;
        } catch (err) {
            console.error('Error checking RCON connection:', err);
            setIsConnected(false);
            return false;
        }
    }, [serverName, isConnected]);

    // Check if server is running
    const checkServerRunning = useCallback(async (): Promise<boolean> => {
        try {
            const isRunning = await invoke<boolean>('is_server_running', { serverName });
            return isRunning;
        } catch (err) {
            console.error('Failed to check server status:', err);
            return false;
        }
    }, [serverName]);

    // Get server RCON password dynamically
    const getServerRconPassword = useCallback(async (): Promise<string> => {
        try {
            const password = await invoke<string>('get_server_rcon_password', { serverName });
            return password;
        } catch (err) {
            console.warn('Failed to get server RCON password, using default:', err);
            return rconConfig.password; // Fallback to provided config
        }
    }, [serverName, rconConfig.password]);

    // Setup RCON configuration
    const setupRcon = useCallback(async (config: RconConfig) => {
        try {
            setError(null);
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
            setError(errorMsg);
            console.error(errorMsg);
            console.error('RCON setup config was:', config);
            throw err;
        }
    }, [serverName]);

    // Connect to RCON (simplified)
    const connect = useCallback(async () => {
        if (isConnecting) return;
        
        setIsConnecting(true);
        setError(null);

        try {
            // Get dynamic RCON password
            const password = await getServerRconPassword();
            const dynamicConfig = { ...rconConfig, password };
            
            // Setup RCON configuration with dynamic password
            await setupRcon(dynamicConfig);
            
            // Try to connect - backend handles persistence and reconnection
            console.log(`Attempting RCON connection to ${serverName}...`);
            await invoke('connect_rcon', { serverName });
            
            setIsConnected(true);
            console.log(`✅ Connected to RCON server: ${serverName}`);
        } catch (err) {
            const errorMsg = `Failed to connect to RCON: ${err}`;
            setError(errorMsg);
            setIsConnected(false);
            console.error(errorMsg);
            throw err;
        } finally {
            setIsConnecting(false);
        }
    }, [serverName, rconConfig, setupRcon, isConnecting, getServerRconPassword]);

    // Disconnect from RCON
    const disconnect = useCallback(async () => {
        try {
            await invoke('disconnect_rcon', { serverName });
            setIsConnected(false);
            setError(null);
            console.log(`Disconnected from RCON server: ${serverName}`);
        } catch (err) {
            const errorMsg = `Failed to disconnect from RCON: ${err}`;
            setError(errorMsg);
            console.error(errorMsg);
        }
    }, [serverName]);

    // Execute RCON command (let backend handle reconnection)
    const executeCommand = useCallback(async (command: string): Promise<string> => {
        try {
            setError(null);
            const response = await invoke<string>('execute_rcon_command', {
                serverName,
                command
            });
            
            // Update connection status on successful command
            if (!isConnected) {
                setIsConnected(true);
            }
            
            return response;
        } catch (err) {
            const errorMsg = `Failed to execute RCON command: ${err}`;
            setError(errorMsg);
            console.error(errorMsg);
            
            // Update connection status on error
            setIsConnected(false);
            
            throw err;
        }
    }, [serverName, isConnected]);

    // Test connection
    const testConnection = useCallback(async (): Promise<boolean> => {
        try {
            setError(null);
            const result = await invoke<boolean>('test_rcon_connection', { serverName });
            setIsConnected(result);
            return result;
        } catch (err) {
            const errorMsg = `Failed to test RCON connection: ${err}`;
            setError(errorMsg);
            setIsConnected(false);
            console.error(errorMsg);
            return false;
        }
    }, [serverName]);

    // Auto-connect on mount if enabled (but only once)
    useEffect(() => {
        let hasAttempted = false;
        
        if (autoConnect && !hasAttempted) {
            hasAttempted = true;
            console.log(`Attempting auto-connect for server: ${serverName}`);
            
            // Simple auto-connect without frequent checking
            checkConnectionStatus().then(connected => {
                if (!connected && !isConnecting) {
                    console.log(`Not connected to ${serverName}, starting connection process...`);
                    connect().catch(err => {
                        console.error('Auto-connect failed:', err);
                        // Backend will handle reconnections automatically
                    });
                } else {
                    console.log(`Already connected or connecting to ${serverName}`);
                }
            });
        }
    }, [serverName, autoConnect]); // Removed other dependencies to prevent re-runs

    // Check connection status periodically (less frequent)
    useEffect(() => {
        // Reduced frequency - let backend handle reconnections
        const interval = setInterval(() => {
            checkConnectionStatus();
        }, isConnecting ? 3000 : 15000); // 3s when connecting, 15s otherwise

        return () => clearInterval(interval);
    }, [isConnecting, checkConnectionStatus]);

    return {
        isConnected,
        isConnecting,
        error,
        connect,
        disconnect,
        executeCommand,
        setupRcon,
        testConnection
    };
};