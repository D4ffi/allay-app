import React, { ReactNode } from 'react';
import { X } from 'lucide-react';
import { ToolTip } from '../common/ToolTip.tsx';

interface ModalProps {
    isOpen: boolean;
    onClose: () => void;
    title?: string;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    children: ReactNode;
}

export const AddServermodel = ({
    isOpen, 
    onClose, 
    title = "Modal", 
    size = 'md', 
    children 
}: ModalProps) => {
    if (!isOpen) return null;

    const sizeClasses = {
        sm: 'max-w-md',
        md: 'max-w-lg', 
        lg: 'max-w-2xl',
        xl: 'max-w-4xl'
    };

    const handleOverlayClick = (e: React.MouseEvent) => {
        if (e.target === e.currentTarget) {
            onClose();
        }
    };

    return (
        <div 
            className="fixed inset-0 z-50 flex items-center justify-center p-4"
            style={{ backgroundColor: 'rgba(0, 0, 0, 0.5)' }}
            onClick={handleOverlayClick}
        >
            <div 
                className={`
                    bg-white rounded-lg shadow-xl w-full ${sizeClasses[size]}
                    max-h-[90vh] overflow-hidden
                    transform transition-all duration-200 ease-out
                    animate-in zoom-in-95 fade-in-0
                `}
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-gray-200">
                    <h2 className="text-lg font-semibold text-gray-900">
                        {title}
                    </h2>
                    <ToolTip content="Close" position="bottom" delay={300}>
                        <button
                            onClick={onClose}
                            className="p-1 rounded hover:bg-red-500/50 cursor-pointer transition-colors"
                        >
                            <X size={20} />
                        </button>
                    </ToolTip>
                </div>

                {/* Content */}
                <div className="p-4 overflow-y-auto max-h-[calc(90vh-80px)]">
                    {children}
                </div>
            </div>
        </div>
    );
};