import { useState } from 'react';
import { ChevronRight, ChevronLeft, Plus, Search, Pin, Filter, Settings2 } from 'lucide-react';
import { ToolTip } from './ToolTip';
import { CreateServerModal } from '../modals/CreateServerModal';

interface ActionBarProps {
    onCreateServer: (serverData: any) => void;
}

export const ActionBar = ({ onCreateServer }: ActionBarProps) => {
    const [isExpanded, setIsExpanded] = useState(true);
    const [isModalOpen, setIsModalOpen] = useState(false);

    const EXPANSION_DURATION = 300;
    const ICON_DELAY = 200;
    const ICON_ANIMATION_DURATION = 200;

    const actionIcons = [
        { icon: Plus, delay: ICON_DELAY * 0.7, tooltip: "Create", onClick: () => setIsModalOpen(true) },
        { icon: Search, delay: ICON_DELAY * 0.65, tooltip: "Search", onClick: () => {} },
        { icon: Pin, delay: ICON_DELAY * 0.6, tooltip: "Pin", onClick: () => {} },
        { icon: Filter, delay: ICON_DELAY * 0.55, tooltip: "Filter", onClick: () => {} },
        { icon: Settings2, delay: ICON_DELAY * 0.5, tooltip: "Settings", onClick: () => {}}
    ];

    const toggleExpansion = () => {
        setIsExpanded(!isExpanded);
    };

    return (
        <div className="absolute left-2 z-40 h-[100px] flex items-center">
            <div
                className={`flex items-center transition-all ease-in-out h-full`}
                style={{
                    width: isExpanded ? '300px' : '48px',
                    transitionDuration: `${EXPANSION_DURATION}ms`
                }}
            >
                <div className={`flex items-center space-x-2 h-full ${
                    !isExpanded ? 'overflow-hidden' : ''
                }`}>
                {actionIcons.map(({ icon: Icon, delay, tooltip, onClick }, index) => (
                        <div
                            key={index}
                            className={`
                                transition-opacity ease-out
                                ${isExpanded ? 'opacity-100' : 'opacity-0 pointer-events-none'}
                            `}
                            style={{
                                transitionDelay: isExpanded ? `${delay}ms` : `${delay}ms`,
                                transitionDuration: `${ICON_ANIMATION_DURATION}ms`
                            }}
                        >
                            <ToolTip
                                content={tooltip}
                                position="bottom"
                                delay={300}
                            >
                                <button
                                    className="ml-2 p-2 rounded hover:bg-gray-400/50 cursor-pointer"
                                    onClick={onClick}
                                >
                                    <Icon size={16} />
                                </button>
                            </ToolTip>
                        </div>
                    ))}
                </div>

                <ToolTip
                    content={isExpanded ? "Collapse" : "Expand"}
                    position="bottom"
                    delay={400}
                >
                    <button
                        onClick={toggleExpansion}
                        className={`
                            p-2 rounded hover:bg-gray-400/50 cursor-pointer
                            transition-all ease-in-out
                            ${isExpanded ? 'ml-2' : 'ml-0'}
                        `}
                        style={{ transitionDuration: `${EXPANSION_DURATION}ms` }}
                    >
                        {isExpanded ? <ChevronLeft size={16} /> : <ChevronRight size={16} />}
                    </button>
                </ToolTip>
            </div>

            <CreateServerModal 
                isOpen={isModalOpen} 
                onClose={() => setIsModalOpen(false)}
                onCreateServer={onCreateServer}
            />
        </div>
    );
}