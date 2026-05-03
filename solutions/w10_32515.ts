// src/AppCore.tsx
import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { isTauri } from './utils/environment';

const AppCore: React.FC = () => {
  const navigate = useNavigate();
  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initialize = async () => {
      try {
        // Only attempt Tauri-specific initialization if running in Tauri
        if (isTauri()) {
          const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
          const appWindow = getCurrentWebviewWindow();
          
          // Listen for window events if needed
          await appWindow.onResized(() => {
            // Handle resize
          });
        }

        // Continue with normal initialization
        setInitialized(true);
      } catch (err) {
        console.error('Failed to initialize:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
      }
    };

    initialize();
  }, []);

  if (error) {
    return (
      <div className="error-screen">
        <h2>Failed to Initialize</h2>
        <p>{error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  if (!initialized) {
    return <div className="loading-screen">Loading...</div>;
  }

  return (
    <div className="app-core">
      {/* Welcome UI content */}
      <h1>Welcome to Cortex</h1>
      <button onClick={() => navigate('/editor')}>Start Coding</button>
    </div>
  );
};

export default AppCore;
