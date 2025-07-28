import React, { useState, useEffect, useRef } from 'react';
import { RefreshCw } from 'lucide-react';
import { useRconContext } from '../../contexts/RconContext';
import { useServerState } from '../../contexts/ServerStateContext';
import { TerminalEditor } from './TerminalEditor';

interface TerminalProps {
    serverName: string;
}

interface TerminalLine {
    id: string;
    content: string;
    type: 'output' | 'command' | 'error' | 'system';
    timestamp: Date;
}

export const Terminal = ({ serverName }: TerminalProps) => {
    const [lines, setLines] = useState<TerminalLine[]>([]);
    const [currentCommand, setCurrentCommand] = useState('');
    const [commandHistory, setCommandHistory] = useState<string[]>([]);
    const [historyIndex, setHistoryIndex] = useState(-1);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);

    // RCON integration via context (simplified)
    const rconContext = useRconContext();

    // Server state integration
    const serverState = useServerState();
    const serverStatus = serverState.getServerStatus(serverName);


    // Focus input when the component mounts
    useEffect(() => {
        if (inputRef.current) {
            inputRef.current.focus();
        }
    }, []);

    // Add a welcome message on the mount
    useEffect(() => {
        const welcomeLine: TerminalLine = {
            id: `welcome-${Date.now()}`,
            content: `Terminal instance for: ${serverName}`,
            type: 'system',
            timestamp: new Date()
        };
        setLines([welcomeLine]);
    }, [serverName]);

    // No auto-connection or status monitoring needed

    // Internal command handlers
    const executeInternalCommand = async (command: string): Promise<string> => {
        const parts = command.split(' ');
        const cmd = parts[0].toLowerCase();
        const args = parts.slice(1);

        switch (cmd) {
            case 'a-clear':
                // Create a welcome line again
                const welcomeLine: TerminalLine = {
                    id: `welcome-${Date.now()}`,
                    content: `Terminal instance for: ${serverName}`,
                    type: 'system',
                    timestamp: new Date()
                };
                setLines([welcomeLine]);
                return 'Terminal cleared';

            case 'a-connect':
                return 'RCON connections are now managed automatically per command.';

            case 'a-start':
                try {
                    if (serverStatus === 'online') {
                        return `Server '${serverName}' is already running`;
                    }
                    if (serverStatus === 'starting') {
                        return `Server '${serverName}' is already starting`;
                    }
                    
                    await serverState.startServer(serverName);
                    return `Starting server '${serverName}'...`;
                } catch (error) {
                    return `Failed to start server: ${error}`;
                }

            case 'a-stop':
                try {
                    if (serverStatus === 'offline') {
                        return `Server '${serverName}' is already stopped`;
                    }
                    if (serverStatus === 'stopping') {
                        return `Server '${serverName}' is already stopping`;
                    }
                    
                    await serverState.stopServer(serverName, true); // graceful stop
                    return `Stopping server '${serverName}' gracefully...`;
                } catch (error) {
                    return `Failed to stop server: ${error}`;
                }

            case 'a-help':
                return `Available internal commands: \n
a-clear - Clear the terminal screen
a-connect - Attempt to connect to RCON server
a-start - Start the Minecraft server
a-stop - Stop the Minecraft server gracefully
a-help - Show this help message

All other commands are sent to the Minecraft server via RCON.`;

            default:
                return `Unknown internal command: ${cmd}. Type 'ahelp' for available commands.`;
        }
    };

    const handleCommandSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        
        if (!currentCommand.trim()) return;

        const command = currentCommand.trim();

        // Add command to lines
        const commandLine: TerminalLine = {
            id: `cmd-${Date.now()}`,
            content: `> ${command}`,
            type: 'command',
            timestamp: new Date()
        };

        setLines(prev => [...prev, commandLine]);

        // Add to history
        setCommandHistory(prev => [...prev, command]);
        setHistoryIndex(-1);
        
        // Clear input
        setCurrentCommand('');

        // Check if it's an internal command (starts with 'a')
        if (command.toLowerCase().startsWith('a')) {
            try {
                const response = await executeInternalCommand(command);
                
                const responseLine: TerminalLine = {
                    id: `resp-${Date.now()}`,
                    content: response,
                    type: 'system',
                    timestamp: new Date()
                };
                
                setLines(prev => [...prev, responseLine]);
            } catch (error) {
                const errorLine: TerminalLine = {
                    id: `error-${Date.now()}`,
                    content: `Internal command error: ${error}`,
                    type: 'error',
                    timestamp: new Date()
                };
                
                setLines(prev => [...prev, errorLine]);
            }
        } else {
            // Execute command via RCON (ephemeral connection)
            try {
                const response = await rconContext.executeCommand(serverName, command);
                
                const responseLine: TerminalLine = {
                    id: `resp-${Date.now()}`,
                    content: response || 'Command executed successfully (no output)',
                    type: 'output',
                    timestamp: new Date()
                };
                
                setLines(prev => [...prev, responseLine]);
            } catch (error) {
                const errorLine: TerminalLine = {
                    id: `error-${Date.now()}`,
                    content: 'Server is not running or unreachable',
                    type: 'error',
                    timestamp: new Date()
                };
                
                setLines(prev => [...prev, errorLine]);
            }
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'ArrowUp') {
            e.preventDefault();
            if (commandHistory.length > 0) {
                const newIndex = historyIndex === -1 ? commandHistory.length - 1 : Math.max(0, historyIndex - 1);
                setHistoryIndex(newIndex);
                setCurrentCommand(commandHistory[newIndex]);
            }
        } else if (e.key === 'ArrowDown') {
            e.preventDefault();
            if (historyIndex !== -1) {
                const newIndex = Math.min(commandHistory.length - 1, historyIndex + 1);
                if (newIndex === commandHistory.length - 1 && historyIndex === commandHistory.length - 1) {
                    setHistoryIndex(-1);
                    setCurrentCommand('');
                } else {
                    setHistoryIndex(newIndex);
                    setCurrentCommand(commandHistory[newIndex]);
                }
            }
        }
    };

    // Handle refresh (clear terminal)
    const handleRefresh = async () => {
        if (isRefreshing) return;
        
        setIsRefreshing(true);
        
        // Clear terminal and add welcome message
        const welcomeLine: TerminalLine = {
            id: `welcome-${Date.now()}`,
            content: `Terminal instance for: ${serverName}`,
            type: 'system',
            timestamp: new Date()
        };
        setLines([welcomeLine]);
        
        setTimeout(() => setIsRefreshing(false), 500);
    };

    return (
        <div className="h-full flex flex-col  border border-gray-700 rounded-lg overflow-hidden">
            {/* Terminal Header */}
            <div className="bg-gray-600 border-b border-gray-700 px-4 py-2 flex items-center justify-between flex-shrink-0">
                <div className="text-gray-300 text-sm font-medium">
                    Terminal - {serverName}
                </div>
                <button
                    onClick={handleRefresh}
                    disabled={isRefreshing}
                    className={`p-1.5 rounded hover:bg-gray-500 transition-colors duration-200 ${
                        isRefreshing 
                            ? 'text-gray-500 cursor-not-allowed' 
                            : 'text-gray-300 hover:text-white'
                    }`}
                    title="Clear terminal"
                >
                    <RefreshCw 
                        size={16} 
                        className={isRefreshing ? 'animate-spin' : ''} 
                    />
                </button>
            </div>

            {/* Terminal Editor - Contains the scrollable content */}
            <TerminalEditor lines={lines} serverName={serverName} />

            {/* Command Input */}
            <div className="bg-gray-700 border-t border-gray-700 p-3 flex-shrink-0">
                <form onSubmit={handleCommandSubmit} className="flex items-center space-x-2">
                    <span className="font-mono text-sm shrink-0 text-green-400">
                        $
                    </span>
                    <input
                        ref={inputRef}
                        type="text"
                        value={currentCommand}
                        onChange={(e) => setCurrentCommand(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Enter Minecraft command..."
                        className="flex-1 bg-transparent font-mono text-sm outline-none text-gray-300 placeholder-gray-500"
                        spellCheck={false}
                        autoComplete="off"
                    />
                </form>
            </div>
        </div>
    );
};