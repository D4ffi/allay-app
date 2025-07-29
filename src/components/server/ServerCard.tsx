import React from "react";
import { Play, Square, Edit, Folder, Trash2 } from 'lucide-react';
import { ContextMenu } from '../common/ContextMenu';
import { MinecraftMOTD } from '../common/MinecraftMOTD';
import { useLocale } from '../../contexts/LocaleContext';
import { useServerState } from '../../contexts/ServerStateContext';

interface ServerCardProps {
    name: string;
    description: string;
    hasCustomImg: boolean;
    imgUrl: string;
    serverType: string;
    version: string;
    loaderVersion: string;
    playerCount?: number;
    maxPlayers?: number;
    onEdit?: () => void;
    onOpenFolder?: () => void;
    onDelete?: () => void;
    onClick?: () => void;
}

export const ServerCard: React.FC<ServerCardProps> = ({
    name,
    description,
    hasCustomImg,
    imgUrl,
    serverType,
    version = "1.21",
    loaderVersion,
    playerCount = 0,
    maxPlayers = 20,
    onEdit,
    onOpenFolder,
    onDelete,
    onClick
}) => {
    const { t } = useLocale();
    const serverState = useServerState();
    
    // Estado simple: solo online u offline (v2)
    const status = serverState.getServerStatus(name);
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

    const handleStartStop = async () => {
        try {
            if (isOnline) {
                await serverState.stopServer(name);
            } else {
                await serverState.startServer(name);
            }
        } catch (error) {
            console.error(`Failed to ${isOnline ? 'stop' : 'start'} server ${name}:`, error);
        }
    };

    const contextMenuItems = [
        {
            label: isOnline ? t('serverCard.stopServer') : t('serverCard.startServer'),
            icon: isOnline ? Square : Play,
            onClick: handleStartStop,
        },
        {
            label: t('serverCard.editServer'),
            icon: Edit,
            onClick: () => onEdit?.(),
        },
        {
            label: t('serverCard.openInExplorer'),
            icon: Folder,
            onClick: () => onOpenFolder?.(),
        },
        {
            label: t('serverCard.deleteServer'),
            icon: Trash2,
            onClick: () => onDelete?.(),
            destructive: true,
        },
    ];

    return (
        <ContextMenu items={contextMenuItems}>
            <div 
                className="bg-background rounded-lg shadow-sm border p-4 mb-4 cursor-pointer transition-colors server-card-border"
                onClick={onClick}
            >
            <div className="flex items-start justify-between">
                <div className="flex items-start space-x-4">
                    {/* Server Icon */}
                    <div className="w-16 h-16 rounded-lg overflow-hidden">
                        <img 
                            src={hasCustomImg ? imgUrl : (isOnline ? "/profile.png" : "/profile-off.png")}
                            alt={`${name} server icon`}
                            className="w-full h-full object-cover"
                        />
                    </div>

                    {/* Server Info */}
                    <div className="flex-1">
                        <h3 className="text-lg font-semibold text-text mb-1">{name}</h3>
                        <div className="mb-3">
                            <MinecraftMOTD 
                                motd={description} 
                                theme="light"
                                className="text-sm leading-relaxed"
                            />
                        </div>

                        {/* Tags */}
                        <div className="flex flex-wrap gap-2">
                            <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: getLoaderBadgeColor(serverType), color: 'var(--color-text-loader-badge)'}}>
                                {serverType}
                            </span>
                            <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: 'var(--color-version-badge)', color: 'var(--color-text-version-badge)'}}>
                                {version}
                            </span>
                            {loaderVersion && (
                                <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: getLoaderBadgeColor(serverType), color: 'var(--color-text-loader-badge)'}}>
                                    {loaderVersion}
                                </span>
                            )}
                            {isOnline ? (
                                <span className="px-2 py-1 text-xs rounded-full" style={{backgroundColor: 'var(--color-online-badge)', color: 'var(--color-text-online-badge)'}}>
                                    {t('serverCard.running')}
                                </span>
                            ) : (
                                <span className="px-2 py-1 bg-surface text-text-muted text-xs rounded-full">
                                    {t('serverCard.offline')}
                                </span>
                            )}
                        </div>
                    </div>
                </div>

                {/* Controls and Player Count */}
                <div className="flex flex-col items-end space-y-3">
                    {/* Start/Stop Button */}
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            handleStartStop();
                        }}
                        className={`
                            flex items-center justify-center gap-2 px-4 py-2 rounded-lg transition-all duration-200 font-medium text-sm
                            ${isOnline 
                                ? 'bg-danger hover:bg-danger-hover text-white shadow-lg' 
                                : 'bg-success hover:bg-success-hover text-white shadow-lg'
                            }
                            hover:scale-105 active:scale-95 hover:cursor-pointer
                        `}
                        title={isOnline ? t('serverCard.stopServer') : t('serverCard.startServer')}
                    >
                        {isOnline ? (
                            <>
                                <Square size={16} fill="currentColor" />
                                Stop
                            </>
                        ) : (
                            <>
                                <Play size={16} fill="currentColor" className="ml-0.5" />
                                Run
                            </>
                        )}
                    </button>

                    {/* Player Count */}
                    <div className="text-right">
                        <span className="text-lg font-semibold text-text">
                            {playerCount}/{maxPlayers}
                        </span>
                        <p className="text-xs text-text-muted">
                            {t('serverCard.players')}
                        </p>
                    </div>
                </div>
            </div>
            </div>
        </ContextMenu>
    );
};