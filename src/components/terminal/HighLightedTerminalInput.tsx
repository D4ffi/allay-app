import React, { useRef, useEffect, useState } from 'react';

interface HighlightedTerminalInputProps {
    value: string;
    onChange: (value: string) => void;
    onKeyDown: (e: React.KeyboardEvent) => void;
    onSubmit: (e: React.FormEvent) => void;
    placeholder?: string;
    disabled?: boolean;
}

export const HighlightedTerminalInput = ({
                                             value,
                                             onChange,
                                             onKeyDown,
                                             onSubmit,
                                             placeholder = "Enter Minecraft command...",
                                             disabled = false
                                         }: HighlightedTerminalInputProps) => {
    const inputRef = useRef<HTMLInputElement>(null);
    useRef<HTMLDivElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [, setCursorPosition] = useState(0);

    // Focus input when the component mounts
    useEffect(() => {
        if (inputRef.current) {
            inputRef.current.focus();
        }
    }, []);

    // Update cursor position
    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const newValue = e.target.value;
        onChange(newValue);
        setCursorPosition(e.target.selectionStart || 0);
    };

    // Handle key events and update the cursor position
    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        onKeyDown(e);

        // Update the cursor position after a brief delay to get the correct position
        setTimeout(() => {
            if (inputRef.current) {
                setCursorPosition(inputRef.current.selectionStart || 0);
            }
        }, 0);
    };

    // Handle click to update the cursor position
    const handleClick = () => {
        if (inputRef.current) {
            setCursorPosition(inputRef.current.selectionStart || 0);
        }
    };

    // Parse command to get first word and rest
    const parseCommand = (command: string) => {
        const trimmed = command.trim();
        if (!trimmed) return { firstWord: '', rest: '', fullCommand: command };

        const firstSpaceIndex = trimmed.indexOf(' ');
        if (firstSpaceIndex === -1) {
            return {
                firstWord: trimmed,
                rest: '',
                fullCommand: command
            };
        }

        const firstWord = trimmed.substring(0, firstSpaceIndex);
        const rest = command.substring(command.indexOf(firstWord) + firstWord.length);

        return { firstWord, rest, fullCommand: command };
    };

    parseCommand(value);

    // Create highlighted text for display
    const createHighlightedText = () => {
        if (!value) return null;

        // Split the value to preserve exact spacing
        const parts = [];
        let currentIndex = 0;

        // Find the first non-whitespace character
        const trimStart = value.search(/\S/);

        if (trimStart === -1) {
            // Only whitespace
            return (
                <div className="pointer-events-none absolute inset-0 flex items-center whitespace-pre">
                    <span className="font-mono text-sm text-transparent select-none">
                        {value}
                    </span>
                </div>
            );
        }

        // Leading whitespace
        if (trimStart > 0) {
            parts.push(
                <span key="leading" className="font-mono text-sm text-transparent select-none">
                    {value.substring(0, trimStart)}
                </span>
            );
            currentIndex = trimStart;
        }

        // Find the first word (non-whitespace sequence)
        const restOfString = value.substring(currentIndex);
        const firstWordMatch = restOfString.match(/^\S+/);

        if (firstWordMatch) {
            const firstWordEnd = currentIndex + firstWordMatch[0].length;

            // First word in yellow
            parts.push(
                <span key="firstword" className="font-mono text-sm text-yellow-400 select-none">
                    {value.substring(currentIndex, firstWordEnd)}
                </span>
            );

            // Rest of the string in gray
            if (firstWordEnd < value.length) {
                parts.push(
                    <span key="rest" className="font-mono text-sm text-gray-300 select-none">
                        {value.substring(firstWordEnd)}
                    </span>
                );
            }
        }

        return (
            <div className="pointer-events-none absolute inset-0 flex items-center whitespace-pre">
                {parts}
            </div>
        );
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onSubmit(e);
    };

    const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            const formEvent = new Event('submit', { bubbles: true, cancelable: true }) as any;
            handleSubmit(formEvent);
        }
        handleKeyDown(e);
    };

    return (
        <div className="flex items-center space-x-2">
            <span className="font-mono text-sm shrink-0 text-green-400">
                $
            </span>
            <div
                ref={containerRef}
                className="flex-1 relative"
            >
                {/* Highlighted text overlay */}
                {createHighlightedText()}

                {/* Actual input (transparent text) */}
                <input
                    ref={inputRef}
                    type="text"
                    value={value}
                    onChange={handleInputChange}
                    onKeyDown={handleKeyPress}
                    onClick={handleClick}
                    onFocus={handleClick}
                    placeholder={placeholder}
                    disabled={disabled}
                    className="w-full bg-transparent font-mono text-sm outline-none text-transparent placeholder-gray-500 caret-gray-300"
                    style={{
                        caretColor: '#d1d5db', // Gray-300 for the cursor
                        letterSpacing: 'normal'
                    }}
                    spellCheck={false}
                    autoComplete="off"
                />

                {/* Show a placeholder when input is empty */}
                {!value && (
                    <div className="pointer-events-none absolute inset-0 flex items-center">
                        <span className="font-mono text-sm text-gray-500 select-none whitespace-pre">
                            {placeholder}
                        </span>
                    </div>
                )}
            </div>
        </div>
    );
};