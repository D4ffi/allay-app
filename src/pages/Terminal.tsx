
import React from 'react';
import { ArrowLeft } from 'lucide-react';
import { AllayLayout } from "../components/common/AllayLayout";
import { Terminal as TerminalComponent } from "../components/terminal/Terminal";
import { useLocale } from '../contexts/LocaleContext';
import { useServerState } from '../contexts/ServerStateContext';

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
    memory?: number;
}

interface TerminalPageProps {
    server: Server;
    onBack: () => void;
}

const TerminalPage = ({ server, onBack }: TerminalPageProps) => {
    useLocale();
    const serverState = useServerState();
    
    // Estado simple: solo online u offline
    const status = serverState.getServerStatus(server.name);
    const isOnline = status === 'online';

    return (
        <div className="h-screen pt-12 flex flex-col overflow-hidden bg-surface">
            <AllayLayout title={`Terminal - ${server.name}`} />
            
            {/* Header with Back Button */}
            <div className="p-4 flex items-center space-x-4 border-b border-border bg-background flex-shrink-0">
                <button
                    onClick={onBack}
                    className="p-2 rounded hover:bg-surface-hover transition-colors flex-shrink-0 text-text"
                >
                    <ArrowLeft size={20} />
                </button>
                
                {/* Server Info */}
                <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 rounded overflow-hidden flex-shrink-0">
                        <img 
                            src={server.hasCustomImg ? server.imgUrl : (isOnline ? "/profile.png" : "/profile-off.png")}
                            alt={`${server.name} server icon`}
                            className="w-full h-full object-cover"
                        />
                    </div>
                    <div>
                        <h1 className="text-lg font-semibold text-text">
                            {server.name} Terminal
                        </h1>
                        <div className="flex items-center space-x-2">
                            <span className={`w-2 h-2 rounded-full ${
                                isOnline ? 'bg-success' : 'bg-text-muted'
                            }`}></span>
                            <span className="text-sm text-text-secondary">
                                {isOnline ? 'Online' : 'Offline'}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Terminal Component */}
            <div className="flex-1 p-4 bg-surface overflow-hidden">
                <div className="h-full max-w-7xl mx-auto">
                    <TerminalComponent serverName={server.name} />
                </div>
            </div>
        </div>
    );
};

export default TerminalPage;
