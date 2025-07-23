import { ArrowLeft } from 'lucide-react';
import { AllayLayout } from "../components/common/AllayLayout";
import { MinecraftMOTD } from "../components/common/MinecraftMOTD";
import { useLocale } from '../contexts/LocaleContext';

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

const ServerDetails = ({ server, onBack }: ServerDetailsProps) => {
    const { t } = useLocale();

    return (
        <div className="h-screen pt-8">
            <AllayLayout title="Server Details" />
            
            {/* Header with Mini Server Card */}
            <div className="p-4 pt-12 flex items-center space-x-4">
                <button
                    onClick={onBack}
                    className="p-2 rounded hover:bg-gray-200 transition-colors flex-shrink-0"
                >
                    <ArrowLeft size={20} />
                </button>
                
                {/* Mini Server Card */}
                <div className="flex items-center space-x-3 min-w-0">
                    {/* Server Icon */}
                    <div className="w-12 h-12 rounded-lg overflow-hidden flex-shrink-0">
                        <img 
                            src={server.hasCustomImg ? server.imgUrl : (server.isOnline ? "/profile.png" : "/profile-off.png")}
                            alt={`${server.name} server icon`}
                            className="w-full h-full object-cover"
                        />
                    </div>

                    {/* Server Info */}
                    <div className="flex-1 min-w-0">
                        <h1 className="text-xl font-bold text-gray-900 truncate">
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
                            <span className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded-full">
                                {server.serverType}
                            </span>
                            <span className="px-2 py-1 bg-blue-100 text-blue-700 text-xs rounded-full">
                                {server.version}
                            </span>
                            {server.loaderVersion && (
                                <span className="px-2 py-1 bg-purple-100 text-purple-700 text-xs rounded-full">
                                    {server.loaderVersion}
                                </span>
                            )}
                            {server.isOnline ? (
                                <span className="px-2 py-1 bg-green-100 text-green-700 text-xs rounded-full">
                                    Online
                                </span>
                            ) : (
                                <span className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded-full">
                                    Offline
                                </span>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            {/* Server Details Content */}
            <div className="p-4 max-w-4xl mx-auto space-y-6">
                
                {/* Placeholder content */}
                <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                    <h2 className="text-lg font-semibold text-gray-900 mb-4">
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