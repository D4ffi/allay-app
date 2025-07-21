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
                    <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center">
                        <AlertTriangle className="w-8 h-8 text-red-600" />
                    </div>
                    
                    <div className="space-y-2">
                        <h3 className="text-lg font-semibold text-gray-900">
                            {t('deleteServerModal.confirmMessage')}
                        </h3>
                        <p className="text-sm text-gray-600">
                            {t('deleteServerModal.warningText')}
                        </p>
                    </div>
                </div>

                {/* Server Info */}
                <div className="bg-gray-50 rounded-lg p-4 border border-gray-200">
                    <div className="flex items-center space-x-4">
                        {/* Server Image */}
                        <div className="flex-shrink-0">
                            <img
                                src={serverImage || '/profile.png'}
                                alt={serverName}
                                className="w-16 h-16 rounded-lg object-cover border-2 border-gray-300"
                                onError={(e) => {
                                    const target = e.target as HTMLImageElement;
                                    target.src = '/profile.png';
                                }}
                            />
                        </div>
                        
                        {/* Server Details */}
                        <div className="flex-1 min-w-0">
                            <h4 className="text-lg font-semibold text-gray-900 truncate">
                                {serverName}
                            </h4>
                            <p className="text-sm text-gray-500">
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
                        className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    >
                        {t('common.cancel')}
                    </button>
                    
                    <button
                        type="button"
                        onClick={handleDelete}
                        disabled={isDeleting}
                        className="flex-1 px-4 py-2 text-sm font-medium text-white bg-red-600 border border-transparent rounded-lg hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center space-x-2"
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