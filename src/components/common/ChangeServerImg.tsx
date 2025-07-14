import { useState, useRef } from 'react';

interface ChangeServerImgProps {
    defaultImage?: string;
    onImageChange?: (imageFile: File | null, imageUrl: string) => void;
    size?: 'sm' | 'md' | 'lg';
    className?: string;
}

export const ChangeServerImg = ({
    defaultImage = '/profile.png',
    onImageChange,
    size = 'md',
    className = ''
}: ChangeServerImgProps) => {
    const [currentImage, setCurrentImage] = useState<string>(defaultImage);
    const fileInputRef = useRef<HTMLInputElement>(null);

    const sizeClasses = {
        sm: 'w-16 h-16',
        md: 'w-24 h-24', 
        lg: 'w-32 h-32'
    };


    const handleImageClick = () => {
        fileInputRef.current?.click();
    };

    const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        const file = event.target.files?.[0];
        
        if (file) {
            // Validar que sea una imagen
            if (!file.type.startsWith('image/')) {
                alert('Please select a valid image file');
                return;
            }

            // Validar tamaño (max 5MB)
            if (file.size > 5 * 1024 * 1024) {
                alert('Image size must be less than 5MB');
                return;
            }

            // Crear URL para preview
            const imageUrl = URL.createObjectURL(file);
            setCurrentImage(imageUrl);
            
            // Callback al padre
            onImageChange?.(file, imageUrl);
        }
    };

    const resetToDefault = () => {
        setCurrentImage(defaultImage);
        onImageChange?.(null, defaultImage);
        
        // Limpiar el input file
        if (fileInputRef.current) {
            fileInputRef.current.value = '';
        }
    };

    return (
        <div className={`relative inline-block ${className}`}>
            {/* Imagen principal clickeable */}
                <div 
                    onClick={handleImageClick}
                    className={`
                        ${sizeClasses[size]} 
                        relative rounded-lg overflow-hidden bg-gray-100 border-2 border-gray-300
                        shadow-sm hover:shadow-md transition-all duration-200 cursor-pointer
                        hover:border-blue-400 hover:scale-105
                    `}
                >
                    <img
                        src={currentImage}
                        alt="Server icon"
                        className="w-full h-full object-cover"
                        onError={() => {
                            // Si falla cargar la imagen, usar la por defecto
                            setCurrentImage(defaultImage);
                        }}
                    />
                    
                    {/* Overlay hover */}
                    <div className="absolute inset-0 hover:bg-black/50 transition-all duration-200 flex items-center justify-center opacity-0 hover:opacity-100">
                        <span className="text-white text-sm font-medium">Click to change</span>
                    </div>

                    {/* Botón de reset (solo visible si no es la imagen por defecto) */}
                    {currentImage !== defaultImage && (
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                resetToDefault();
                            }}
                            className="absolute bottom-2 right-2 w-7 h-7 bg-gray-500 hover:bg-gray-600 text-white rounded-full flex items-center justify-center shadow-lg transition-all duration-200 hover:scale-110 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 z-10"
                        >
                            <span className="text-sm font-bold">×</span>
                        </button>
                    )}
                </div>

            {/* Input de archivo oculto */}
            <input
                ref={fileInputRef}
                type="file"
                accept="image/*"
                onChange={handleFileChange}
                className="hidden"
            />
        </div>
    );
};