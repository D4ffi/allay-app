import { useRef, useEffect, useState } from 'react';

interface TerminalLine {
    id: string;
    content: string;
    type: 'output' | 'command' | 'error' | 'system';
    timestamp: Date;
}

interface TerminalEditorProps {
    lines: TerminalLine[];
    serverName: string;
}

export const TerminalEditor = ({ lines}: TerminalEditorProps) => {
    const [autoScroll, setAutoScroll] = useState(true);
    const editorRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to the bottom when new lines are added
    useEffect(() => {
        if (editorRef.current && autoScroll) {
            editorRef.current.scrollTop = editorRef.current.scrollHeight;
        }
    }, [lines, autoScroll]);

    // Handle manual scrolling to detect if user wants to review history
    const handleScroll = () => {
        if (editorRef.current) {
            const { scrollTop, scrollHeight, clientHeight } = editorRef.current;
            const isAtBottom = scrollHeight - scrollTop <= clientHeight + 5; // 5px tolerance
            setAutoScroll(isAtBottom);
        }
    };

    const formatTimestamp = (date: Date) => {
        return date.toLocaleTimeString('en-US', { 
            hour12: false, 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit' 
        });
    };

    const getLineColor = (type: TerminalLine['type']) => {
        switch (type) {
            case 'command':
                return 'text-green-400';
            case 'error':
                return 'text-red-400';
            case 'system':
                return 'text-blue-300';
            case 'output':
            default:
                return 'text-gray-300';
        }
    };

    return (
        <div className="flex-1 relative">
            {/* Terminal Editor Content - This is where the scroll happens */}
            <div 
                ref={editorRef}
                onScroll={handleScroll}
                className="absolute inset-0 p-3 overflow-y-auto font-mono text-sm leading-relaxed terminal-scroll"
                style={{ 
                    backgroundColor: '#2B2B2B', // JetBrains dark theme background
                    scrollbarWidth: 'thin',
                    scrollbarColor: '#4A4A4A #2B2B2B'
                }}
            >
                {lines.map((line) => (
                    <div key={line.id} className="flex items-start space-x-2 mb-1">
                        <span className="text-gray-500 text-xs shrink-0 w-20">
                            {formatTimestamp(line.timestamp)}
                        </span>
                        <span className={`${getLineColor(line.type)} break-all`}>
                            {line.content}
                        </span>
                    </div>
                ))}
            </div>

            {/* Scroll to bottom indicator - Only shown when not auto-scrolling */}
            {!autoScroll && (
                <button
                    onClick={() => {
                        if (editorRef.current) {
                            editorRef.current.scrollTop = editorRef.current.scrollHeight;
                            setAutoScroll(true);
                        }
                    }}
                    className="absolute bottom-4 right-4 bg-blue-600 hover:bg-blue-700 text-white text-xs px-3 py-1 rounded-full shadow-lg transition-colors duration-200 z-10"
                    title="Scroll to bottom"
                >
                    ↓ New messages
                </button>
            )}
        </div>
    );
};