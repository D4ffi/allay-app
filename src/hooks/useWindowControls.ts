import { getCurrentWindow } from '@tauri-apps/api/window';
import { useState, useEffect } from 'react';

export const useWindowControls = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const window = getCurrentWindow();

  useEffect(() => {
    // Verificar estado inicial
    const checkMaximized = async () => {
      try {
        const maximized = await window.isMaximized();
        setIsMaximized(maximized);
      } catch (error) {
        console.error('Failed to check maximize state:', error);
      }
    };

    checkMaximized();

    // Escuchar cambios de estado
    let unlisten: (() => void) | undefined;
    
    const setupListener = async () => {
      try {
        unlisten = await window.onResized(async () => {
          const maximized = await window.isMaximized();
          setIsMaximized(maximized);
        });
      } catch (error) {
        console.error('Failed to setup resize listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [window]);

  const minimize = async () => {
    try {
      await window.minimize();
    } catch (error) {
      console.error('Failed to minimize window:', error);
    }
  };

  const toggleMaximize = async () => {
    try {
      await window.toggleMaximize();
    } catch (error) {
      console.error('Failed to toggle maximize window:', error);
    }
  };

  const close = async () => {
    try {
      await window.close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  };

  const startDrag = async () => {
    try {
      await window.startDragging();
    } catch (error) {
      console.error('Failed to start dragging:', error);
    }
  };

  return { minimize, toggleMaximize, close, startDrag, isMaximized };
};