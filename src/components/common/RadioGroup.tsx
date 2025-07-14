import { useState } from 'react';

interface RadioOption {
    value: string;
    label: string;
    description?: string;
}

interface RadioGroupProps {
    name: string;
    options: RadioOption[];
    value?: string;
    onChange?: (value: string) => void;
    className?: string;
    layout?: 'vertical' | 'horizontal' | 'grid';
}

export const RadioGroup = ({
    name,
    options,
    value,
    onChange,
    className = "",
    layout = 'vertical'
}: RadioGroupProps) => {
    const [selectedValue, setSelectedValue] = useState(value || '');

    const handleChange = (optionValue: string) => {
        setSelectedValue(optionValue);
        onChange?.(optionValue);
    };

    const getLayoutClasses = () => {
        switch (layout) {
            case 'horizontal':
                return 'flex flex-wrap gap-4';
            case 'grid':
                return 'grid grid-cols-2 gap-3';
            case 'vertical':
            default:
                return 'space-y-3';
        }
    };

    return (
        <div className={`${className}`}>
            <div className={getLayoutClasses()}>
                {options.map((option) => {
                    const isSelected = selectedValue === option.value;
                    return (
                        <label
                            key={option.value}
                            className={`
                                flex items-start space-x-3 p-3 rounded-lg border cursor-pointer
                                transition-all duration-200 hover:bg-gray-50
                                ${isSelected 
                                    ? 'border-blue-500 bg-blue-50 ring-1 ring-blue-500' 
                                    : 'border-gray-300 bg-white hover:border-gray-400'
                                }
                            `}
                        >
                            <div className="flex items-center h-5">
                                <input
                                    type="radio"
                                    name={name}
                                    value={option.value}
                                    checked={isSelected}
                                    onChange={() => handleChange(option.value)}
                                    className={`
                                        w-4 h-4 border-2 rounded-full appearance-none cursor-pointer
                                        transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
                                        ${isSelected 
                                            ? 'border-blue-500 bg-blue-500' 
                                            : 'border-gray-300 bg-white hover:border-blue-400'
                                        }
                                    `}
                                    style={{
                                        backgroundImage: isSelected 
                                            ? 'radial-gradient(circle, white 2px, transparent 2px)' 
                                            : 'none'
                                    }}
                                />
                            </div>
                            <div className="flex-1 min-w-0">
                                <div className={`
                                    text-sm font-medium transition-colors duration-200
                                    ${isSelected ? 'text-blue-900' : 'text-gray-900'}
                                `}>
                                    {option.label}
                                </div>
                                {option.description && (
                                    <div className={`
                                        text-xs mt-1 transition-colors duration-200
                                        ${isSelected ? 'text-blue-700' : 'text-gray-500'}
                                    `}>
                                        {option.description}
                                    </div>
                                )}
                            </div>
                        </label>
                    );
                })}
            </div>
        </div>
    );
};