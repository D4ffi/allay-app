import { useState } from 'react';
import { Modal } from '../common/Modal';
import { Trash2, AlertTriangle } from 'lucide-react';
import { useLocale } from '../../contexts/LocaleContext';

interface DeleteServerModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirmDelete: () => void;
    serverName: string;
    serverImage: string;
}

export const DeleteServerModal = ({ 
    isOpen, 
    onClose, 
    onConfirmDelete, 
    serverName,
    serverImage
}: DeleteServerModalProps) => {
    const { t } = useLocale();
    const [isDeleting, setIsDeleting] = useState(false);

    const handleDelete = async () => {
        setIsDeleting(true);
        try {
            await onConfirmDelete();
            onClose();
        } catch (error) {
            console.error('Error deleting server:', error);
        } finally {
            setIsDeleting(false);
        }
    };

    const handleClose = () => {
        if (!isDeleting) {
            onClose();
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={handleClose}
            title={t('deleteServerModal.title')}
            size="md"
        >
            <div className="space-y-6">
                {/* Warning Icon and Message */}
                <div className="flex flex-col items-center text-center space-y-4">
                    <div className="w-16 h-16 bg-danger-light rounded-full flex items-center justify-center">
                        <AlertTriangle className="w-8 h-8 text-danger" />
                    </div>
                    
                    <div className="space-y-2">
                        <h3 className="text-lg font-semibold text-text">
                            {t('deleteServerModal.confirmMessage')}
                        </h3>
                        <p className="text-sm text-text-secondary">
                            {t('deleteServerModal.warningText')}
                        </p>
                    </div>
                </div>

                {/* Server Info */}
                <div className="bg-surface rounded-lg p-4 border border-border">
                    <div className="flex items-center space-x-4">
                        {/* Server Image */}
                        <div className="flex-shrink-0">
                            <img
                                src={serverImage || '/profile.png'}
                                alt={serverName}
                                className="w-16 h-16 rounded-lg object-cover border-2 border-border"
                                onError={(e) => {
                                    const target = e.target as HTMLImageElement;
                                    target.src = '/profile.png';
                                }}
                            />
                        </div>
                        
                        {/* Server Details */}
                        <div className="flex-1 min-w-0">
                            <h4 className="text-lg font-semibold text-text truncate">
                                {serverName}
                            </h4>
                            <p className="text-sm text-text-muted">
                                {t('deleteServerModal.serverFolder')}: storage/{serverName}
                            </p>
                        </div>
                    </div>
                </div>

                {/* Action Buttons */}
                <div className="flex space-x-3 pt-4">
                    <button
                        type="button"
                        onClick={handleClose}
                        disabled={isDeleting}
                        className="hover:cursor-pointer flex-1 px-4 py-2 text-sm font-medium text-text-secondary bg-background border border-border rounded-lg hover:bg-surface focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    >
                        {t('common.cancel')}
                    </button>
                    
                    <button
                        type="button"
                        onClick={handleDelete}
                        disabled={isDeleting}
                        className="hover:cursor-pointer flex-1 px-4 py-2 text-sm font-medium text-white bg-danger border border-transparent rounded-lg hover:bg-danger-hover focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-danger disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center space-x-2"
                    >
                        {isDeleting ? (
                            <>
                                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                                <span>{t('deleteServerModal.deleting')}</span>
                            </>
                        ) : (
                            <>
                                <Trash2 className="w-4 h-4" />
                                <span>{t('deleteServerModal.confirmDelete')}</span>
                            </>
                        )}
                    </button>
                </div>
            </div>
        </Modal>
    );
};