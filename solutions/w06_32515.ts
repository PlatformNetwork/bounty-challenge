// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { isTauri } from './utils/tauriCheck';

// Only import Tauri APIs when running in Tauri environment
let getCurrentWebviewWindow: any = () => null;

if (isTauri()) {
  // Dynamic import to prevent crash in browser
  import('@tauri-apps/api/webviewWindow').then((module) => {
    getCurrentWebviewWindow = module.getCurrentWebviewWindow;
  });
}

const AppCore: React.FC = () => {
  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Check if running in Tauri environment
        if (isTauri()) {
          const appWindow = getCurrentWebviewWindow();
          if (appWindow) {
            // Tauri-specific initialization
            console.log('Running in Tauri environment');
          }
        } else {
          // Browser-specific initialization
          console.log('Running in browser environment');
        }
        
        setInitialized(true);
      } catch (err) {
        console.error('Initialization failed:', err);
        setError('Failed to initialize application');
      }
    };

    initializeApp();
  }, []);

  if (error) {
    return <div className="error-screen">{error}</div>;
  }

  if (!initialized) {
    return <div className="loading-screen">Loading...</div>;
  }

  return (
    <div className="app-core">
      {/* Your app content here */}
      <h1>Welcome to Cortex</h1>
    </div>
  );
};

export default AppCore;
