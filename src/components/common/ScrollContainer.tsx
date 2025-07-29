import React, { ReactNode } from 'react';

interface ScrollContainerProps {
  children: ReactNode;
  className?: string;
  maxHeight?: string;
}

export const ScrollContainer: React.FC<ScrollContainerProps> = ({ 
  children, 
  className = "",
  maxHeight = "100%" 
}) => {
  return (
    <div 
      className={`overflow-y-auto allay-scroll ${className}`}
      style={{ maxHeight }}
    >
      {children}
    </div>
  );
};