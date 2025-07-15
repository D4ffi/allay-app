import React from "react";

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
    maxPlayers = 20
}) => {
    return (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4 mb-4">
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
                        <p className="text-gray-600 text-sm mb-3">{description}</p>

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
                                    running
                                </span>
                            ) : (
                                <span className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded-full">
                                    offline
                                </span>
                            )}
                        </div>
                    </div>
                </div>

                {/* Player Count */}
                <div className="text-right">
                    <span className="text-lg font-semibold text-gray-900">
                        {playerCount}/{maxPlayers}
                    </span>
                </div>
            </div>
        </div>
    );
};
