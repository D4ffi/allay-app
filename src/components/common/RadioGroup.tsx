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
    disabled?: boolean;
}

export const RadioGroup = ({
    name,
    options,
    value,
    onChange,
    className = "",
    layout = 'vertical',
    disabled = false
}: RadioGroupProps) => {
    const [selectedValue, setSelectedValue] = useState(value || '');

    const handleChange = (optionValue: string) => {
        if (disabled) return;
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
                                flex items-start space-x-3 p-3 rounded-lg border transition-all duration-200
                                ${disabled 
                                    ? 'cursor-not-allowed opacity-50 bg-surface border-border' 
                                    : 'cursor-pointer hover:bg-surface-hover'
                                }
                                ${!disabled && isSelected 
                                    ? 'border-primary bg-primary-light ring-1 ring-primary' 
                                    : !disabled 
                                        ? 'border-border bg-background hover:border-border-hover'
                                        : ''
                                }
                            `}
                        >
                            <div className="flex items-center h-5">
                                <input
                                    type="radio"
                                    name={name}
                                    value={option.value}
                                    checked={isSelected}
                                    disabled={disabled}
                                    onChange={() => handleChange(option.value)}
                                    className={`
                                        w-4 h-4 border-2 rounded-full appearance-none transition-all duration-200
                                        ${disabled 
                                            ? 'cursor-not-allowed border-border bg-surface' 
                                            : 'cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2'
                                        }
                                        ${!disabled && isSelected 
                                            ? 'border-primary bg-primary' 
                                            : !disabled 
                                                ? 'border-border bg-background hover:border-primary'
                                                : ''
                                        }
                                    `}
                                    style={{
                                        backgroundImage: isSelected && !disabled
                                            ? 'radial-gradient(circle, white 2px, transparent 2px)' 
                                            : 'none'
                                    }}
                                />
                            </div>
                            <div className="flex-1 min-w-0">
                                <div className={`
                                    text-sm font-medium transition-colors duration-200
                                    ${disabled 
                                        ? 'text-text-muted' 
                                        : isSelected 
                                            ? 'text-primary' 
                                            : 'text-text'
                                    }
                                `}>
                                    {option.label}
                                </div>
                                {option.description && (
                                    <div className={`
                                        text-xs mt-1 transition-colors duration-200
                                        ${disabled 
                                            ? 'text-text-muted' 
                                            : isSelected 
                                                ? 'text-primary' 
                                                : 'text-text-secondary'
                                        }
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