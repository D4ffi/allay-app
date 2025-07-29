
import { Minus, Square, X } from 'lucide-react';
import { useWindowControls } from '../../hooks/useWindowControls';
import { ToolTip } from './ToolTip';

interface AllayLayoutProps {
    title?: string;
}

export const AllayLayout = ({ title = "Home" }: AllayLayoutProps) => {
    const { minimize, toggleMaximize, close, startDrag } = useWindowControls();

    return(
        <div 
            className="fixed top-0 left-0 w-full h-12 bg-transparent flex items-center justify-between px-4 z-50"
            onMouseDown={startDrag}
        >
            {/* Logo a la izquierda */}
            <div className="flex items-center pointer-events-none gap-2">
                <img 
                    src="/profile.png" 
                    alt="Allay" 
                    className="w-6 h-6"
                />
                <p className="text-text font-medium">{title}</p>
            </div>
            
            {/* Controles de ventana a la derecha */}
            <div 
                className="flex items-center space-x-2 pointer-events-auto"
                onMouseDown={(e) => e.stopPropagation()}
            >
                <ToolTip content="Minimize" position="bottom" delay={300}>
                    <button 
                        className="p-2 hover:bg-surface-hover rounded cursor-pointer text-text-secondary hover:text-text transition-colors"
                        onClick={minimize}
                    >
                        <Minus size={16} />
                    </button>
                </ToolTip>
                <ToolTip content="Maximize" position="bottom" delay={300}>
                    <button 
                        className="p-2 hover:bg-surface-hover rounded cursor-pointer text-text-secondary hover:text-text transition-colors"
                        onClick={toggleMaximize}
                    >
                        <Square size={16} />
                    </button>
                </ToolTip>
                <ToolTip content="Close" position="bottom" delay={300}>
                    <button 
                        className="p-2 hover:bg-danger/50 rounded cursor-pointer text-text-secondary hover:text-white transition-colors"
                        onClick={close}
                    >
                        <X size={16} />
                    </button>
                </ToolTip>
            </div>
        </div>
    )
}

