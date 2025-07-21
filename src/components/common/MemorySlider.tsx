import { AlertTriangle } from 'lucide-react';
import { ToolTip } from './ToolTip';
import { useSystemInfo } from '../../contexts/SystemContext';

interface MemorySliderProps {
    value: number; // Value in MB
    onChange: (value: number) => void;
    disabled?: boolean;
    className?: string;
}

export const MemorySlider = ({ value, onChange, disabled = false, className = '' }: MemorySliderProps) => {
    const { systemMemoryMB: maxMemoryMB, isMemoryLoading, memoryError } = useSystemInfo();

    const formatMemory = (mb: number): string => {
        if (mb < 1024) {
            return `${mb} MB`;
        }
        
        const gb = mb / 1024;
        if (gb < 10) {
            return `${gb.toFixed(1)} GB`;
        }
        return `${Math.round(gb)} GB`;
    };

    const getMemoryPercentage = (): number => {
        return (value / maxMemoryMB) * 100;
    };

    const isOverThreshold = (): boolean => {
        return getMemoryPercentage() > 75;
    };


    // Calculate slider steps (every 256MB for smooth sliding)
    const step = 256;
    const min = 512; // Minimum 512MB
    const max = maxMemoryMB;

    return (
        <div className={`space-y-4 ${className}`}>
            {/* Header with title and warning */}
            <div className="flex items-center justify-between">
                <label className="block text-sm font-semibold text-gray-800">
                    Memory Allocation
                </label>
                
                {isOverThreshold() && (
                    <ToolTip 
                        content="Warning: Allocating more than 75% of system memory may cause performance issues. Consider leaving more memory available for your operating system."
                        position="left"
                        delay={100}
                    >
                        <div className="flex items-center text-red-600">
                            <AlertTriangle size={18} className="animate-pulse" />
                        </div>
                    </ToolTip>
                )}
            </div>

            {/* Memory info and error handling */}
            {isMemoryLoading ? (
                <div className="flex items-center space-x-2 text-sm text-gray-500">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-gray-400"></div>
                    <span>Detecting system memory...</span>
                </div>
            ) : memoryError ? (
                <div className="space-y-1">
                    <div className="text-sm text-red-600 font-medium">
                        Memory detection failed
                    </div>
                    <div className="text-xs text-gray-500">
                        Using fallback: {formatMemory(maxMemoryMB)} limit
                    </div>
                </div>
            ) : (
                <div className="text-sm text-gray-600">
                    System Memory: {formatMemory(maxMemoryMB)}
                </div>
            )}

            {/* Current value display */}
            <div className="flex items-center justify-between">
                <span className="text-lg font-semibold text-gray-900">
                    {formatMemory(value)}
                </span>
                <span className={`text-sm font-medium ${isOverThreshold() ? 'text-red-600' : 'text-gray-600'}`}>
                    {getMemoryPercentage().toFixed(1)}% of system memory
                </span>
            </div>

            {/* Slider */}
            <div className="relative">
                <input
                    type="range"
                    min={min}
                    max={max}
                    step={step}
                    value={value}
                    onChange={(e) => onChange(parseInt(e.target.value))}
                    disabled={disabled || isMemoryLoading}
                    className={`
                        w-full h-3 rounded-lg appearance-none cursor-pointer transition-all duration-200
                        bg-gray-200
                        ${disabled || isMemoryLoading ? 'opacity-50 cursor-not-allowed' : ''}
                        memory-slider
                    `}
                    style={{
                        background: `linear-gradient(to right, 
                            ${isOverThreshold() ? '#dc2626' : '#3b82f6'} 0%, 
                            ${isOverThreshold() ? '#dc2626' : '#3b82f6'} ${getMemoryPercentage()}%, 
                            #e5e7eb ${getMemoryPercentage()}%, 
                            #e5e7eb 100%)`
                    }}
                />
                
                {/* Slider styling */}
                <style dangerouslySetInnerHTML={{
                    __html: `
                        .memory-slider::-webkit-slider-thumb {
                            appearance: none;
                            height: 20px;
                            width: 20px;
                            border-radius: 50%;
                            background: ${isOverThreshold() ? '#dc2626' : '#3b82f6'};
                            cursor: pointer;
                            border: 2px solid white;
                            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
                            transition: all 0.2s ease;
                        }
                        
                        .memory-slider::-webkit-slider-thumb:hover {
                            transform: scale(1.1);
                            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
                        }
                        
                        .memory-slider::-moz-range-thumb {
                            height: 20px;
                            width: 20px;
                            border-radius: 50%;
                            background: ${isOverThreshold() ? '#dc2626' : '#3b82f6'};
                            cursor: pointer;
                            border: 2px solid white;
                            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
                            transition: all 0.2s ease;
                        }
                        
                        .memory-slider::-webkit-slider-track {
                            height: 12px;
                            border-radius: 6px;
                            background: transparent;
                        }
                        
                        .memory-slider::-moz-range-track {
                            height: 12px;
                            border-radius: 6px;
                            background: transparent;
                            border: none;
                        }
                    `
                }} />
            </div>

            {/* Memory range indicators */}
            <div className="flex justify-between text-xs text-gray-500">
                <span>Min: {formatMemory(min)}</span>
                <span className="font-medium">75%: {formatMemory(Math.round(maxMemoryMB * 0.75))}</span>
                <span>Max: {formatMemory(max)}</span>
            </div>

            {/* Quick preset buttons */}
            <div className="flex space-x-2">
                <button
                    onClick={() => onChange(1024)}
                    disabled={disabled || isMemoryLoading}
                    className={`px-3 py-1 text-xs rounded-md border transition-all duration-200 ${
                        value === 1024
                            ? 'bg-blue-100 text-blue-700 border-blue-300'
                            : 'bg-white text-gray-600 border-gray-300 hover:border-gray-400'
                    } ${disabled || isMemoryLoading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
                >
                    1GB
                </button>
                <button
                    onClick={() => onChange(2048)}
                    disabled={disabled || isMemoryLoading}
                    className={`px-3 py-1 text-xs rounded-md border transition-all duration-200 ${
                        value === 2048
                            ? 'bg-blue-100 text-blue-700 border-blue-300'
                            : 'bg-white text-gray-600 border-gray-300 hover:border-gray-400'
                    } ${disabled || isMemoryLoading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
                >
                    2GB
                </button>
                <button
                    onClick={() => onChange(4096)}
                    disabled={disabled || isMemoryLoading}
                    className={`px-3 py-1 text-xs rounded-md border transition-all duration-200 ${
                        value === 4096
                            ? 'bg-blue-100 text-blue-700 border-blue-300'
                            : 'bg-white text-gray-600 border-gray-300 hover:border-gray-400'
                    } ${disabled || isMemoryLoading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
                >
                    4GB
                </button>
                <button
                    onClick={() => onChange(Math.round(maxMemoryMB * 0.5))}
                    disabled={disabled || isMemoryLoading}
                    className="px-3 py-1 text-xs rounded-md border bg-white text-gray-600 border-gray-300 hover:border-gray-400 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                >
                    50%
                </button>
            </div>
        </div>
    );
};