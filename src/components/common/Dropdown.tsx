import { useState, useRef, useEffect } from 'react';
import { ChevronDown } from 'lucide-react';

interface DropdownOption {
    value: string;
    label: string;
}

interface DropdownProps {
    options: DropdownOption[];
    placeholder?: string;
    value?: string;
    onChange?: (value: string) => void;
    disabled?: boolean;
    className?: string;
}

export const Dropdown = ({
    options,
    placeholder = "Select an option...",
    value,
    onChange,
    disabled = false,
    className = ""
}: DropdownProps) => {
    const [isOpen, setIsOpen] = useState(false);
    const [selectedValue, setSelectedValue] = useState(value || '');
    const dropdownRef = useRef<HTMLDivElement>(null);

    // Close dropdown when clicking outside
    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    // Update selected value when value prop changes
    useEffect(() => {
        if (value !== undefined) {
            setSelectedValue(value);
        }
    }, [value]);

    const handleSelect = (optionValue: string) => {
        setSelectedValue(optionValue);
        setIsOpen(false);
        onChange?.(optionValue);
    };

    const toggleDropdown = () => {
        if (!disabled) {
            setIsOpen(!isOpen);
        }
    };

    const selectedOption = options.find(option => option.value === selectedValue);
    const displayText = selectedOption ? selectedOption.label : placeholder;

    return (
        <div className={`relative ${className}`} ref={dropdownRef}>
            {/* Trigger Button */}
            <button
                type="button"
                onClick={toggleDropdown}
                disabled={disabled}
                className={`
                    w-full px-3 py-2 text-left bg-white border border-gray-300 rounded-md 
                    focus:outline-none focus:ring-2 focus:ring-gray-400 focus:border-transparent
                    flex items-center justify-between
                    transition-colors duration-200
                    ${disabled ? 'bg-gray-100 cursor-not-allowed' : 'hover:bg-gray-50 cursor-pointer'}
                    ${isOpen ? 'ring-2 ring-gray-400 border-transparent' : ''}
                `}
            >
                <span className={`block truncate ${!selectedValue ? 'text-gray-500' : 'text-gray-900'}`}>
                    {displayText}
                </span>
                <ChevronDown 
                    size={16} 
                    className={`
                        ml-2 transition-transform duration-200 flex-shrink-0
                        ${isOpen ? 'rotate-180' : 'rotate-0'}
                        ${disabled ? 'text-gray-400' : 'text-gray-600'}
                    `}
                />
            </button>

            {/* Dropdown Menu */}
            {isOpen && !disabled && (
                <div 
                    className="absolute z-50 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg max-h-80 overflow-y-auto"
                    style={{
                        scrollbarWidth: 'thin',
                        scrollbarColor: '#D1D5DB #F3F4F6'
                    }}
                >
                    {options.length === 0 ? (
                        <div className="px-3 py-2 text-gray-500 text-sm">
                            No options available
                        </div>
                    ) : (
                        options.map((option) => (
                            <button
                                key={option.value}
                                type="button"
                                onClick={() => handleSelect(option.value)}
                                className={`
                                    w-full px-3 py-2 text-left text-sm transition-colors duration-150
                                    hover:bg-gray-100 focus:bg-gray-100 focus:outline-none
                                    ${selectedValue === option.value ? 'bg-blue-50 text-blue-700' : 'text-gray-900'}
                                `}
                            >
                                {option.label}
                            </button>
                        ))
                    )}
                </div>
            )}
        </div>
    );
};