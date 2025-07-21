import React from "react";
import { Play, Square, Edit, Folder, Trash2 } from 'lucide-react';
import { ContextMenu } from '../common/ContextMenu';
import { MinecraftMOTD } from '../common/MinecraftMOTD';
import { useLocale } from '../../contexts/LocaleContext';

interface ServerCardProps {
    name: string;
    description: string;
    hasCustomImg: boolean;
    imgUrl: string;
    serverType: string;
    version: string;
    loaderVersion: string;
    isOnline?: boolean;
    playerCount?: number;
    maxPlayers?: number;
    onStartStop?: () => void;
    onEdit?: () => void;
    onOpenFolder?: () => void;
    onDelete?: () => void;
}

export const ServerCard: React.FC<ServerCardProps> = ({
    name,
    description,
    hasCustomImg,
    imgUrl,
    serverType,
    version = "1.21",
    loaderVersion,
    isOnline = false,
    playerCount = 0,
    maxPlayers = 20,
    onStartStop,
    onEdit,
    onOpenFolder,
    onDelete
}) => {
    const { t } = useLocale();
    const contextMenuItems = [
        {
            label: isOnline ? t('serverCard.stopServer') : t('serverCard.startServer'),
            icon: isOnline ? Square : Play,
            onClick: () => onStartStop?.(),
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
            <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4 mb-4 cursor-pointer">
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
                        <h3 className="text-lg font-semibold text-gray-900 mb-1">{name}</h3>
                        <div className="mb-3">
                            <MinecraftMOTD 
                                motd={description} 
                                theme="light"
                                className="text-sm leading-relaxed"
                            />
                        </div>

                        {/* Tags */}
                        <div className="flex flex-wrap gap-2">
                            <span className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded-full">
                                {serverType}
                            </span>
                            <span className="px-2 py-1 bg-blue-100 text-blue-700 text-xs rounded-full">
                                {version}
                            </span>
                            {loaderVersion && (
                                <span className="px-2 py-1 bg-purple-100 text-purple-700 text-xs rounded-full">
                                    {loaderVersion}
                                </span>
                            )}
                            {isOnline ? (
                                <span className="px-2 py-1 bg-green-100 text-green-700 text-xs rounded-full">
                                    {t('serverCard.running')}
                                </span>
                            ) : (
                                <span className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded-full">
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
                            onStartStop?.();
                        }}
                        className={`
                            flex items-center justify-center gap-2 px-4 py-2 rounded-lg transition-all duration-200 font-medium text-sm
                            ${isOnline 
                                ? 'bg-red-500 hover:bg-red-600 text-white shadow-lg' 
                                : 'bg-green-500 hover:bg-green-600 text-white shadow-lg'
                            }
                            hover:scale-105 active:scale-95
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
                        <span className="text-lg font-semibold text-gray-900">
                            {playerCount}/{maxPlayers}
                        </span>
                        <p className="text-xs text-gray-500">
                            {t('serverCard.players')}
                        </p>
                    </div>
                </div>
            </div>
            </div>
        </ContextMenu>
    );
};
