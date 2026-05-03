// src/AppCore.tsx
import React, { Suspense, lazy, useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';
import { isTauri } from './utils/isTauri';

// Conditionally import Tauri APIs only when in Tauri environment
const getTauriWindow = () => {
  if (isTauri()) {
    return import('@tauri-apps/api/window').then(mod => mod.getCurrentWebviewWindow());
  }
  return Promise.resolve(null);
};

const AppCore: React.FC = () => {
  const [tauriWindow, setTauriWindow] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const location = useLocation();

  useEffect(() => {
    const init = async () => {
      try {
        const window = await getTauriWindow();
        setTauriWindow(window);
      } catch (error) {
        console.warn('Failed to initialize Tauri window:', error);
      } finally {
        setIsLoading(false);
      }
    };
    init();
  }, []);

  if (isLoading) {
    return <div>Loading...</div>;
  }

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <div className="app-core">
        {/* Your app content here */}
        {tauriWindow && <TauriSpecificComponent window={tauriWindow} />}
      </div>
    </Suspense>
  );
};

export default AppCore;
