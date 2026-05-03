// src/AppCore.tsx
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

// Check if running in Tauri environment
const isTauri = () => {
  try {
    return window.__TAURI_INTERNALS__ !== undefined;
  } catch {
    return false;
  }
};

// Lazy import Tauri API only when in Tauri environment
const getTauriWindow = async () => {
  if (isTauri()) {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    return getCurrentWebviewWindow();
  }
  return null;
};

export const AppCore = () => {
  const navigate = useNavigate();
  const [appWindow, setAppWindow] = useState(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Initialize Tauri window only if in Tauri environment
        if (isTauri()) {
          const window = await getTauriWindow();
          setAppWindow(window);
        }

        // Set up window event listeners (only in Tauri)
        if (appWindow) {
          appWindow.onResized(() => {
            // Handle resize
          });
        }

        setIsInitialized(true);
      } catch (err) {
        console.error('Failed to initialize app:', err);
        setError(err);
      }
    };

    initializeApp();
  }, []);

  if (error) {
    return (
      <div className="error-screen">
        <h2>Failed to Initialize</h2>
        <p>{error.message}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  if (!isInitialized) {
    return <div className="loading-screen">Loading...</div>;
  }

  return (
    <div className="app-container">
      {/* Your app content here */}
      <h1>Welcome to Cortex</h1>
    </div>
  );
};

export default AppCore;
