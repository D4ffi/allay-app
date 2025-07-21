import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';

// Import translation files
import enTranslations from '../locales/en.json';
import esLATranslations from '../locales/es-LA.json';

type SupportedLocale = 'en' | 'es-LA';

interface LocaleContextType {
    locale: SupportedLocale;
    setLocale: (locale: SupportedLocale) => void;
    t: (key: string, params?: Record<string, string | number>) => string;
    availableLocales: { code: SupportedLocale; name: string; nativeName: string }[];
}

const translations = {
    'en': enTranslations,
    'es-LA': esLATranslations,
};

const availableLocales = [
    { code: 'en' as SupportedLocale, name: 'English', nativeName: 'English' },
    { code: 'es-LA' as SupportedLocale, name: 'Spanish (Latin America)', nativeName: 'Español (Latinoamérica)' },
];

const LocaleContext = createContext<LocaleContextType | undefined>(undefined);

interface LocaleProviderProps {
    children: ReactNode;
}

export const LocaleProvider: React.FC<LocaleProviderProps> = ({ children }) => {
    const [locale, setLocaleState] = useState<SupportedLocale>('en');

    useEffect(() => {
        // Load saved locale from localStorage
        const savedLocale = localStorage.getItem('allay-locale') as SupportedLocale;
        if (savedLocale && availableLocales.some(l => l.code === savedLocale)) {
            setLocaleState(savedLocale);
        } else {
            // Try to detect browser language
            const browserLang = navigator.language;
            if (browserLang.startsWith('es')) {
                setLocaleState('es-LA');
            }
        }
    }, []);

    const setLocale = (newLocale: SupportedLocale) => {
        setLocaleState(newLocale);
        localStorage.setItem('allay-locale', newLocale);
    };

    // Translation function with nested key support and parameter interpolation
    const t = (key: string, params?: Record<string, string | number>): string => {
        const keys = key.split('.');
        let value: any = translations[locale];

        for (const k of keys) {
            if (value && typeof value === 'object' && k in value) {
                value = value[k];
            } else {
                // Fallback to English if key not found
                value = translations['en'];
                for (const fallbackKey of keys) {
                    if (value && typeof value === 'object' && fallbackKey in value) {
                        value = value[fallbackKey];
                    } else {
                        console.warn(`Translation key not found: ${key}`);
                        return key; // Return the key itself if not found
                    }
                }
                break;
            }
        }

        if (typeof value !== 'string') {
            console.warn(`Translation key "${key}" does not point to a string value`);
            return key;
        }

        // Replace parameters in the translation
        if (params) {
            return value.replace(/\{\{(\w+)\}\}/g, (match, paramKey) => {
                return params[paramKey]?.toString() || match;
            });
        }

        return value;
    };

    const value: LocaleContextType = {
        locale,
        setLocale,
        t,
        availableLocales,
    };

    return (
        <LocaleContext.Provider value={value}>
            {children}
        </LocaleContext.Provider>
    );
};

export const useLocale = (): LocaleContextType => {
    const context = useContext(LocaleContext);
    if (context === undefined) {
        throw new Error('useLocale must be used within a LocaleProvider');
    }
    return context;
};