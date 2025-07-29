import { useState, useEffect } from 'react';
import { ArrowLeft } from 'lucide-react';
import { AllayLayout } from "../components/common/AllayLayout";
import { MinecraftMOTD } from "../components/common/MinecraftMOTD";
import { useLocale } from '../contexts/LocaleContext';
import { useServerState } from '../contexts/ServerStateContext';
import { NavTittleButton } from "../components/navbar/NavTittleButton.tsx";
import TerminalPage from "./Terminal";

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

interface ServerDetailsProps {
    server: Server;
    onBack: () => void;
}

type ServerDetailsTab = 'overview' | 'terminal' | 'manage' | 'settings';

const ServerDetails = ({ server, onBack }: ServerDetailsProps) => {
    useLocale();
    const [activeTab, setActiveTab] = useState<ServerDetailsTab>('overview');
    const serverState = useServerState();
    
    // Estado simple: solo online u offline
    const status = serverState.getServerStatus(server.name);
    const isOnline = status === 'online';

    // Function to get loader-specific badge color
    const getLoaderBadgeColor = (loader: string) => {
        switch (loader.toLowerCase()) {
            case 'vanilla':
                return 'var(--color-vanilla-badge)';
            case 'fabric':
                return 'var(--color-fabric-badge)';
            case 'forge':
                return 'var(--color-forge-badge)';
            case 'neoforge':
                return 'var(--color-neoforge-badge)';
            case 'paper':
                return 'var(--color-paper-badge)';
            case 'quilt':
                return 'var(--color-quilt-badge)';
            default:
                return 'var(--color-vanilla-badge)';
        }
    };

    // If terminal tab is active, render terminal page
    if (activeTab === 'terminal') {
        return (
            <TerminalPage 
                server={server} 
                onBack={() => setActiveTab('overview')} 
            />
        );
    }

    return (
        <div className="h-screen bg-surface">
            <AllayLayout title="Server Details" />
            
            {/* Header with Mini Server Card */}
            <div className="p-4 flex items-center space-x-6">
                <button
                    onClick={onBack}
                    className="p-2 rounded hover:bg-surface-hover transition-colors flex-shrink-0 hover:cursor-pointer text-text"
                >
                    <ArrowLeft size={20} />
                </button>
                
                {/* Mini Server Card */}
                <div className="flex pt-10 items-center space-x-3 min-w-0">
                    {/* Server Icon */}
                    <div className="w-18 h-18 rounded-lg overflow-hidden flex-shrink-0">
                        <img 
                            src={server.hasCustomImg ? server.imgUrl : (isOnline ? "/profile.png" : "/profile-off.png")}
                            alt={`${server.name} server icon`}
                            className="w-full h-full object-cover"
                        />
                    </div>

                    {/* Server Info */}
                    <div className="flex-1 min-w-0">
                        <h1 className="text-xl font-bold text-text truncate">
                            {server.name}
                        </h1>
                        
                        {/* MOTD */}
                        <div className="mb-2 max-w-md">
                            <MinecraftMOTD 
                                motd={server.description} 
                                theme="light"
                                className="text-sm leading-relaxed"
                            />
                        </div>

                        {/* Tags */}
                        <div className="flex flex-wrap gap-2">
                            <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: getLoaderBadgeColor(server.serverType), color: 'var(--color-text-loader-badge)'}}>
                                {server.serverType}
                            </span>
                            <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: 'var(--color-version-badge)', color: 'var(--color-text-version-badge)'}}>
                                {server.version}
                            </span>
                            {server.loaderVersion && (
                                <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: getLoaderBadgeColor(server.serverType), color: 'var(--color-text-loader-badge)'}}>
                                    {server.loaderVersion}
                                </span>
                            )}
                            {isOnline ? (
                                <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: 'var(--color-online-badge)', color: 'var(--color-text-online-badge)'}}>
                                    Online
                                </span>
                            ) : (
                                <span className="px-2 py-1 bg-surface text-text-muted text-xs rounded-full">
                                    Offline
                                </span>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            {/* Navigation Bar */}
            <div className="px-4 pb-4">
                <nav className="flex justify-center items-center space-x-16 max-w-2xl mx-auto">
                    <NavTittleButton 
                        translationKey="overview"
                        onClick={() => setActiveTab('overview')}
                        isActive={activeTab === 'overview'}
                    />
                    <NavTittleButton 
                        translationKey="terminal"
                        onClick={() => setActiveTab('terminal')}
                        isActive={activeTab === 'terminal'}
                    />
                    <NavTittleButton 
                        translationKey="manage"
                        onClick={() => setActiveTab('manage')}
                        isActive={activeTab === 'manage'}
                    />
                    <NavTittleButton 
                        translationKey="settings"
                        onClick={() => setActiveTab('settings')}
                        isActive={activeTab === 'settings'}
                    />
                </nav>
            </div>

            {/* Server Details Content */}
            <div className="p-4 max-w-4xl mx-auto space-y-6">

                {/* Placeholder content */}
                <div className="bg-background rounded-lg shadow-sm border border-border p-6">
                    <h2 className="text-lg font-semibold text-text mb-4">
                        Server Information
                    </h2>
                    <p className="text-gray-500 text-sm">
                        Server details content will be added here...
                    </p>
                </div>
            </div>
        </div>
    );
};

export default ServerDetails;