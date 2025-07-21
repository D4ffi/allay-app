import { useState, useEffect, useRef } from 'react';

interface MinecraftMOTDProps {
    motd: string;
    className?: string;
    theme?: 'dark' | 'light'; // Theme for background compatibility
}

interface MOTDSegment {
    text: string;
    color?: string;
    bold?: boolean;
    italic?: boolean;
    underline?: boolean;
    strikethrough?: boolean;
    obfuscated?: boolean;
}

// Minecraft color codes for dark theme (original)
const COLOR_CODES_DARK: Record<string, string> = {
    '0': '#000000', // Black
    '1': '#0000AA', // Dark Blue
    '2': '#00AA00', // Dark Green
    '3': '#00AAAA', // Dark Aqua
    '4': '#AA0000', // Dark Red
    '5': '#AA00AA', // Dark Purple
    '6': '#FFAA00', // Gold
    '7': '#AAAAAA', // Gray
    '8': '#555555', // Dark Gray
    '9': '#5555FF', // Blue
    'a': '#55FF55', // Green
    'b': '#55FFFF', // Aqua
    'c': '#FF5555', // Red
    'd': '#FF55FF', // Light Purple
    'e': '#FFFF55', // Yellow
    'f': '#FFFFFF', // White
};

// Minecraft color codes adapted for light theme
const COLOR_CODES_LIGHT: Record<string, string> = {
    '0': '#1f2937', // Dark Gray (instead of black)
    '1': '#1e40af', // Blue
    '2': '#059669', // Green
    '3': '#0891b2', // Cyan
    '4': '#dc2626', // Red
    '5': '#9333ea', // Purple
    '6': '#d97706', // Orange/Gold
    '7': '#6b7280', // Gray
    '8': '#374151', // Dark Gray
    '9': '#3b82f6', // Light Blue
    'a': '#10b981', // Light Green
    'b': '#06b6d4', // Light Cyan
    'c': '#ef4444', // Light Red
    'd': '#a855f7', // Light Purple
    'e': '#eab308', // Yellow
    'f': '#1f2937', // Dark (instead of white)
};

// Format codes
const FORMAT_CODES = {
    'l': 'bold',
    'o': 'italic',
    'n': 'underline',
    'm': 'strikethrough',
    'k': 'obfuscated',
    'r': 'reset'
};

// Random characters for obfuscated text
const OBFUSCATED_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()';

export const MinecraftMOTD = ({ motd, className = '', theme = 'dark' }: MinecraftMOTDProps) => {
    const [parsedSegments, setParsedSegments] = useState<MOTDSegment[]>([]);
    const [obfuscatedText, setObfuscatedText] = useState<Record<number, string>>({});
    const intervalRefs = useRef<Record<number, NodeJS.Timeout>>({});

    // Get color codes based on theme
    const getColorCodes = () => theme === 'light' ? COLOR_CODES_LIGHT : COLOR_CODES_DARK;

    useEffect(() => {
        const segments = parseMOTD(motd, getColorCodes());
        setParsedSegments(segments);
        
        // Clear existing intervals
        Object.values(intervalRefs.current).forEach(clearInterval);
        intervalRefs.current = {};
        
        // Start obfuscation for obfuscated segments
        segments.forEach((segment, index) => {
            if (segment.obfuscated && segment.text) {
                startObfuscation(index, segment.text);
            }
        });

        return () => {
            // Cleanup intervals on unmount
            Object.values(intervalRefs.current).forEach(clearInterval);
        };
    }, [motd, theme]);

    const parseMOTD = (text: string, colorCodes: Record<string, string>): MOTDSegment[] => {
        const segments: MOTDSegment[] = [];
        let currentSegment: MOTDSegment = { text: '' };
        
        let i = 0;
        while (i < text.length) {
            if (text[i] === '§' && i + 1 < text.length) {
                const code = text[i + 1].toLowerCase();
                
                // If we have accumulated text, push current segment
                if (currentSegment.text) {
                    segments.push({ ...currentSegment });
                    currentSegment = { 
                        text: '',
                        color: currentSegment.color,
                        bold: currentSegment.bold,
                        italic: currentSegment.italic,
                        underline: currentSegment.underline,
                        strikethrough: currentSegment.strikethrough,
                        obfuscated: currentSegment.obfuscated
                    };
                }
                
                if (colorCodes[code]) {
                    currentSegment.color = colorCodes[code];
                    // Color codes reset formatting
                    currentSegment.bold = false;
                    currentSegment.italic = false;
                    currentSegment.underline = false;
                    currentSegment.strikethrough = false;
                    currentSegment.obfuscated = false;
                } else if (FORMAT_CODES[code as keyof typeof FORMAT_CODES]) {
                    const format = FORMAT_CODES[code as keyof typeof FORMAT_CODES];
                    if (format === 'reset') {
                        currentSegment = { text: '' };
                    } else {
                        (currentSegment as any)[format] = true;
                    }
                }
                
                i += 2; // Skip the § and the code
            } else {
                currentSegment.text += text[i];
                i++;
            }
        }
        
        // Push the last segment if it has content
        if (currentSegment.text) {
            segments.push(currentSegment);
        }
        
        return segments;
    };

    const startObfuscation = (segmentIndex: number, originalText: string) => {
        const interval = setInterval(() => {
            const obfuscated = originalText
                .split('')
                .map(char => char === ' ' ? ' ' : OBFUSCATED_CHARS[Math.floor(Math.random() * OBFUSCATED_CHARS.length)])
                .join('');
            
            setObfuscatedText(prev => ({
                ...prev,
                [segmentIndex]: obfuscated
            }));
        }, 50); // Change every 50ms for smooth obfuscation effect
        
        intervalRefs.current[segmentIndex] = interval;
    };

    const getSegmentStyle = (segment: MOTDSegment): React.CSSProperties => {
        const defaultColor = theme === 'light' ? '#374151' : '#FFFFFF'; // Gray for light, white for dark
        
        return {
            color: segment.color || defaultColor,
            fontWeight: segment.bold ? 'bold' : 'normal',
            fontStyle: segment.italic ? 'italic' : 'normal',
            textDecoration: [
                segment.underline ? 'underline' : '',
                segment.strikethrough ? 'line-through' : ''
            ].filter(Boolean).join(' ') || 'none',
        };
    };

    const renderSegment = (segment: MOTDSegment, index: number) => {
        const style = getSegmentStyle(segment);
        const displayText = segment.obfuscated 
            ? (obfuscatedText[index] || segment.text)
            : segment.text;

        return (
            <span key={index} style={style}>
                {displayText}
            </span>
        );
    };

    return (
        <div className={`font-mono text-base leading-relaxed ${className}`}>
            {parsedSegments.map((segment, index) => renderSegment(segment, index))}
        </div>
    );
};