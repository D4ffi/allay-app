import React, { createContext, useContext, useEffect, useState } from 'react';

export type Theme = 'allay' | 'allay-dark' | 'enderman' | 'emerald-obsidian' | 'copper-golem';

interface ThemeContextType {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

interface ThemeProviderProps {
  children: React.ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  const [theme, setTheme] = useState<Theme>(() => {
    // Check if there's a saved theme in localStorage
    const savedTheme = localStorage.getItem('allay-theme') as Theme;
    if (savedTheme && (savedTheme === 'allay' || savedTheme === 'allay-dark' || savedTheme === 'enderman' || savedTheme === 'emerald-obsidian' || savedTheme === 'copper-golem')) {
      return savedTheme;
    }
    
    // Default to light theme
    return 'allay';
  });

  useEffect(() => {
    // Apply theme to document
    document.documentElement.setAttribute('data-theme', theme);
    
    // Save theme to localStorage
    localStorage.setItem('allay-theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme(prev => {
      if (prev === 'allay') return 'allay-dark';
      if (prev === 'allay-dark') return 'enderman';
      if (prev === 'enderman') return 'emerald-obsidian';
      if (prev === 'emerald-obsidian') return 'copper-golem';
      return 'allay';
    });
  };

  const value: ThemeContextType = {
    theme,
    setTheme,
    toggleTheme,
  };

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  );
};

export const useTheme = (): ThemeContextType => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};