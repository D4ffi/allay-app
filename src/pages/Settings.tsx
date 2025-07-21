import { ArrowLeft } from 'lucide-react';
import { AllayLayout } from "../components/common/AllayLayout";
import { Dropdown } from "../components/common/Dropdown";
import { useLocale } from '../contexts/LocaleContext';

interface SettingsProps {
    onBack: () => void;
}

const Settings = ({ onBack }: SettingsProps) => {
    const { locale, setLocale, availableLocales, t } = useLocale();

    const handleLanguageChange = (selectedLocale: string) => {
        setLocale(selectedLocale as any);
    };

    return (
        <div className="h-screen pt-8">
            <AllayLayout />
            
            {/* Header */}
            <div className="p-4 pt-12 flex items-center space-x-4">
                <button
                    onClick={onBack}
                    className="p-2 rounded hover:bg-gray-200 transition-colors"
                >
                    <ArrowLeft size={20} />
                </button>
                <h1 className="text-2xl font-bold text-gray-900">
                    {t('common.settings')}
                </h1>
            </div>

            {/* Settings Content */}
            <div className="p-4 max-w-2xl mx-auto space-y-6">
                
                {/* Language Section */}
                <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                    <h2 className="text-lg font-semibold text-gray-900 mb-4">
                        Language / Idioma
                    </h2>
                    
                    <div className="space-y-2">
                        <label className="block text-sm font-medium text-gray-700">
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
                        <p className="text-xs text-gray-500">
                            {t('settings.language.description')}
                        </p>
                    </div>
                </div>

                {/* Future Settings Sections */}
                <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                    <h2 className="text-lg font-semibold text-gray-900 mb-4">
                        {t('common.settings')}
                    </h2>
                    <p className="text-gray-500 text-sm">
                        {t('settings.futureSettings')}
                    </p>
                </div>
            </div>
        </div>
    );
};

export default Settings;