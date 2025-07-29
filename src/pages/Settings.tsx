import { ArrowLeft } from 'lucide-react';
import { AllayLayout } from "../components/common/AllayLayout";
import { Dropdown } from "../components/common/Dropdown";
import { useLocale } from '../contexts/LocaleContext';
import { useTheme, Theme } from '../contexts/ThemeContext';

interface SettingsProps {
    onBack: () => void;
}

const Settings = ({ onBack }: SettingsProps) => {
    const { locale, setLocale, availableLocales, t } = useLocale();
    const { theme, setTheme } = useTheme();

    const handleLanguageChange = (selectedLocale: string) => {
        setLocale(selectedLocale as any);
    };

    const handleThemeChange = (selectedTheme: string) => {
        setTheme(selectedTheme as Theme);
    };

    const themeOptions = [
        { value: 'allay', label: 'Light' },
        { value: 'allay-dark', label: 'Dark' }
    ];

    return (
        <div className="h-screen pt-8 bg-surface">
            <AllayLayout />
            
            {/* Header */}
            <div className="p-4 pt-12 flex items-center space-x-4">
                <button
                    onClick={onBack}
                    className="p-2 rounded hover:bg-surface-hover transition-colors"
                >
                    <ArrowLeft size={20} className="text-text" />
                </button>
                <h1 className="text-2xl font-bold text-text">
                    {t('common.settings')}
                </h1>
            </div>

            {/* Settings Content */}
            <div className="p-4 max-w-2xl mx-auto space-y-6">
                
                {/* Appearance Section */}
                <div className="bg-background rounded-lg shadow-sm border border-border p-6">
                    <h2 className="text-lg font-semibold text-text mb-4">
                        Appearance
                    </h2>
                    
                    <div className="space-y-2">
                        <label className="block text-sm font-medium text-text-secondary">
                            Theme
                        </label>
                        <Dropdown
                            value={theme}
                            onChange={handleThemeChange}
                            options={themeOptions}
                            placeholder="Select theme"
                            className="w-full max-w-xs"
                        />
                        <p className="text-xs text-text-muted">
                            Choose between light and dark theme
                        </p>
                    </div>
                </div>

                {/* Language Section */}
                <div className="bg-background rounded-lg shadow-sm border border-border p-6">
                    <h2 className="text-lg font-semibold text-text mb-4">
                        Language / Idioma
                    </h2>
                    
                    <div className="space-y-2">
                        <label className="block text-sm font-medium text-text-secondary">
                            {t('settings.language.title')}
                        </label>
                        <Dropdown
                            value={locale}
                            onChange={handleLanguageChange}
                            options={availableLocales.map(lang => ({
                                value: lang.code,
                                label: `${t(`languages.${lang.code}`)} (${lang.nativeName})`
                            }))}
                            placeholder={t('settings.language.placeholder')}
                            className="w-full max-w-xs"
                        />
                        <p className="text-xs text-text-muted">
                            {t('settings.language.description')}
                        </p>
                    </div>
                </div>

                {/* Future Settings Sections */}
                <div className="bg-background rounded-lg shadow-sm border border-border p-6">
                    <h2 className="text-lg font-semibold text-text mb-4">
                        {t('common.settings')}
                    </h2>
                    <p className="text-text-muted text-sm">
                        {t('settings.futureSettings')}
                    </p>
                </div>
            </div>
        </div>
    );
};

export default Settings;