import { useLocale } from '../../contexts/LocaleContext';

interface NavTittleButtonProps {
    translationKey: string;
    onClick?: () => void;
    isActive?: boolean;
}

export const NavTittleButton = ({ translationKey, onClick, isActive = false }: NavTittleButtonProps) => {
    const { t } = useLocale();
    
    return (
        <button 
            onClick={onClick}
            className={`nav-item px-4 py-2 font-medium text-sm relative transition-all duration-200 hover:scale-105 hover:underline decoration-2 underline-offset-4 hover:cursor-pointer ${
                isActive 
                    ? 'text-primary underline' 
                    : 'text-text-secondary hover:text-text'
            }`}
            style={{textShadow: '0 1px 2px rgba(0, 0, 0, 0.1)'}}
        >
            {t(`serverDetails.navigation.${translationKey}`)}
        </button>
    )
}