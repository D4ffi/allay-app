import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SystemContextType {
    systemMemoryMB: number;
    isMemoryLoading: boolean;
    memoryError: string | null;
}

const SystemContext = createContext<SystemContextType | undefined>(undefined);

interface SystemProviderProps {
    children: ReactNode;
}

export const SystemProvider: React.FC<SystemProviderProps> = ({ children }) => {
    const [systemMemoryMB, setSystemMemoryMB] = useState(8192); // Default 8GB
    const [isMemoryLoading, setIsMemoryLoading] = useState(true);
    const [memoryError, setMemoryError] = useState<string | null>(null);

    useEffect(() => {
        loadSystemMemory();
    }, []);

    const loadSystemMemory = async () => {
        setIsMemoryLoading(true);
        setMemoryError(null);
        
        try {
            const memoryMB: number = await invoke('get_system_memory_mb');
            
            if (memoryMB && memoryMB > 0) {
                setSystemMemoryMB(memoryMB);
            } else {
                throw new Error('Invalid memory value returned');
            }
        } catch (error) {
            console.error('Error loading system memory:', error);
            setMemoryError(`Failed to detect system memory: ${error}`);
            console.log('Using fallback memory value: 8192 MB');
            // Keep default 8GB if we can't get system memory
            setSystemMemoryMB(8192);
        } finally {
            setIsMemoryLoading(false);
        }
    };

    const value: SystemContextType = {
        systemMemoryMB,
        isMemoryLoading,
        memoryError
    };

    return (
        <SystemContext.Provider value={value}>
            {children}
        </SystemContext.Provider>
    );
};

export const useSystemInfo = (): SystemContextType => {
    const context = useContext(SystemContext);
    if (context === undefined) {
        throw new Error('useSystemInfo must be used within a SystemProvider');
    }
    return context;
};