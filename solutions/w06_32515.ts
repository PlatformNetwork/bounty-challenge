// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

// Lazy load Tauri APIs only when in Tauri environment
const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;

const AppCore: React.FC = () => {
  const navigate = useNavigate();
  const [initialized, setInitialized] = useState(false);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        if (isTauri) {
          // Dynamically import Tauri modules only in Tauri environment
          const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
          const appWindow = getCurrentWebviewWindow();
          
          // Listen for window events only in Tauri
          await appWindow.onResized(() => {
            // Handle resize
          });
        }

        // Initialize app state
        setInitialized(true);
      } catch (error) {
        console.error('App initialization failed:', error);
        // Fallback to browser mode
        setInitialized(true);
      }
    };

    initializeApp();
  }, []);

  if (!initialized) {
    return <div>Loading...</div>;
  }

  return (
    <div className="app-core">
      {/* Your app content */}
      <h1>Welcome to Cortex</h1>
    </div>
  );
};

export default AppCore;
