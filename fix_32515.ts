// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { isTauri } from './utils/tauri';

const AppCore: React.FC = () => {
  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initialize = async () => {
      try {
        if (isTauri()) {
          // Dynamically import Tauri modules only when in Tauri environment
          const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
          const appWindow = getCurrentWebviewWindow();
          // Use appWindow as needed
          console.log('Running in Tauri environment', appWindow);
        } else {
          console.log('Running in browser environment');
        }
        setInitialized(true);
      } catch (err) {
        console.error('Initialization failed:', err);
        setError(err instanceof Error ? err.message : 'Failed to initialize');
      }
    };

    initialize();
  }, []);

  if (error) {
    return <div className="error-screen">Failed to Initialize: {error}</div>;
  }

  if (!initialized) {
    return <div className="loading-screen">Loading...</div>;
  }

  return <div className="welcome-ui">Welcome to Cortex</div>;
};

export default AppCore;
