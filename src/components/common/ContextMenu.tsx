import { useState, useEffect, useRef } from 'react';
import { LucideIcon } from 'lucide-react';

interface ContextMenuItem {
    label: string;
    icon?: LucideIcon;
    onClick: () => void;
    disabled?: boolean;
    destructive?: boolean;
}

interface ContextMenuProps {
    items: ContextMenuItem[];
    children: React.ReactNode;
}

export const ContextMenu = ({ items, children }: ContextMenuProps) => {
    const [isOpen, setIsOpen] = useState(false);
    const [position, setPosition] = useState({ x: 0, y: 0 });
    const menuRef = useRef<HTMLDivElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    const handleContextMenu = (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        
        const rect = containerRef.current?.getBoundingClientRect();
        if (rect) {
            setPosition({
                x: e.clientX,
                y: e.clientY
            });
            setIsOpen(true);
        }
    };

    const handleClick = (item: ContextMenuItem) => {
        if (!item.disabled) {
            item.onClick();
            setIsOpen(false);
        }
    };

    const handleClickOutside = (e: MouseEvent) => {
        if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
            setIsOpen(false);
        }
    };

    useEffect(() => {
        if (isOpen) {
            document.addEventListener('mousedown', handleClickOutside);
            document.addEventListener('scroll', () => setIsOpen(false));
            
            return () => {
                document.removeEventListener('mousedown', handleClickOutside);
                document.removeEventListener('scroll', () => setIsOpen(false));
            };
        }
    }, [isOpen]);

    return (
        <>
            <div
                ref={containerRef}
                onContextMenu={handleContextMenu}
                className="relative"
            >
                {children}
            </div>

            {isOpen && (
                <div
                    ref={menuRef}
                    className="fixed z-50 bg-background border border-border rounded-lg shadow-lg py-1 min-w-[160px]"
                    style={{
                        left: `${position.x}px`,
                        top: `${position.y}px`,
                    }}
                >
                    {items.map((item, index) => {
                        const Icon = item.icon;
                        return (
                            <button
                                key={index}
                                onClick={() => handleClick(item)}
                                disabled={item.disabled}
                                className={`
                                    w-full px-3 py-2 text-left text-sm flex items-center space-x-2
                                    hover:bg-surface-hover transition-colors duration-150
                                    ${item.disabled ? 'text-text-muted cursor-not-allowed' : 'text-text cursor-pointer'}
                                    ${item.destructive ? 'text-danger hover:bg-danger-light' : ''}
                                `}
                            >
                                {Icon && (
                                    <Icon size={16} />
                                )}
                                <span>{item.label}</span>
                            </button>
                        );
                    })}
                </div>
            )}
        </>
    );
};