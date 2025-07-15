import { useState } from 'react';
import {AllayLayout} from "../components/common/AllayLayout.tsx";
import {ActionBar} from "../components/common/ActionBar.tsx";
import { ServerCard } from "../components/server/ServerCard.tsx";

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
}

const Home = () => {
    const [servers, setServers] = useState<Server[]>([]);

    const handleCreateServer = (serverData: Omit<Server, 'id'>) => {
        const newServer: Server = {
            ...serverData,
            id: Date.now().toString() // Simple ID generation
        };
        setServers(prev => [...prev, newServer]);
    };

    return (
        <div className="h-screen pt-8">
            <AllayLayout />
            <ActionBar onCreateServer={handleCreateServer} />

            {servers.length === 0 ? (
                <div className="flex flex-col justify-center items-center h-full gap-4 opacity-30">
                    <img src="/profile-off.png" alt="Allay Off Icon" className="w-30 h-30 drop-shadow-lg drop-shadow-gray-900"/>
                    <p className="text-center text-balance">
                        No server's saved, to create a new one,<br />
                        press the + button in the action bar menu.
                    </p>
                </div>
            ) : (
                <div className="p-4 pt-20 space-y-4 max-w-4xl mx-auto">
                    {servers.map(server => (
                        <ServerCard
                            key={server.id}
                            name={server.name}
                            description={server.description}
                            hasCustomImg={server.hasCustomImg}
                            imgUrl={server.imgUrl}
                            serverType={server.serverType}
                            version={server.version}
                            loaderVersion={server.loaderVersion}
                            isOnline={server.isOnline}
                            playerCount={server.playerCount}
                            maxPlayers={server.maxPlayers}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

export default Home;