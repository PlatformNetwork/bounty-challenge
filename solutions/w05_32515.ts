// src/AppCore.tsx
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

// Lazy load Tauri APIs only when in Tauri environment
const isTauri = () => {
  try {
    return window.__TAURI_INTERNALS__ !== undefined;
  } catch {
    return false;
  }
};

const AppCore: React.FC = () => {
  const navigate = useNavigate();
  const [initialized, setInitialized] = useState(false);

  useEffect(() => {
    const initialize = async () => {
      try {
        if (isTauri()) {
          // Dynamically import Tauri APIs only when in Tauri environment
          const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
          const appWindow = getCurrentWebviewWindow();
          
          // Listen for window events if needed
          appWindow.onResized(() => {
            // Handle resize
          });
        }
        
        setInitialized(true);
      } catch (error) {
        console.error('Failed to initialize:', error);
        // Still set initialized to true to show welcome UI
        setInitialized(true);
      }
    };

    initialize();
  }, []);

  if (!initialized) {
    return null; // Or a loading spinner
  }

  return (
    <div className="app-core">
      {/* Your welcome UI content */}
      <h1>Welcome to Cortex</h1>
      {/* Rest of your app content */}
    </div>
  );
};

export default AppCore;
