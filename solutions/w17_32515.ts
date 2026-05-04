// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { isTauri } from './utils/isTauri';

// Only import Tauri API when running in Tauri environment
let getCurrentWebviewWindow: any = null;
if (isTauri()) {
  // Dynamic import to avoid crashing in browser
  import('@tauri-apps/api/webviewWindow').then(mod => {
    getCurrentWebviewWindow = mod.getCurrentWebviewWindow;
  }).catch(() => {
    // Silently fail if Tauri API is not available
  });
}

const AppCore: React.FC = () => {
  const navigate = useNavigate();
  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Check if running in Tauri environment
        if (isTauri()) {
          // Tauri-specific initialization
          try {
            const webviewWindow = getCurrentWebviewWindow();
            if (webviewWindow) {
              // Perform Tauri-specific setup
              await webviewWindow.setTitle('Cortex');
            }
          } catch (tauriError) {
            console.warn('Tauri initialization failed, continuing in browser mode:', tauriError);
          }
        } else {
          // Browser-specific initialization
          console.log('Running in browser mode');
        }

        // Common initialization logic
        setInitialized(true);
      } catch (err) {
        console.error('Failed to initialize app:', err);
        setError(err instanceof Error ? err.message : 'Unknown error occurred');
      }
    };

    initializeApp();
  }, [navigate]);

  if (error) {
    return (
      <div className="error-screen">
        <h1>Failed to Initialize</h1>
        <p>{error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  if (!initialized) {
    return (
      <div className="loading-screen">
        <div className="spinner" />
        <p>Initializing...</p>
      </div>
    );
  }

  return (
    <div className="app-core">
      {/* Your actual app content here */}
      <WelcomeUI />
    </div>
  );
};

// Placeholder WelcomeUI component
const WelcomeUI: React.FC = () => {
  return (
    <div className="welcome-ui">
      <h1>Welcome to Cortex</h1>
      <p>Your AI-powered development environment</p>
    </div>
  );
};

export default AppCore;
