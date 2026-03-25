To fix the issue, we need to update the `AdminSessions.tsx` file to include the missing stat cards for `averageMessagesPerSession` and `sessionsThisWeek`. Here's the exact code fix:

```typescript
// src/pages/admin/AdminSessions.tsx

import React from 'react';
import { SessionStats } from '../types/admin';

const AdminSessions = () => {
  const [sessionStats, setSessionStats] = React.useState<SessionStats | null>(null);

  const fetchSessionStats = async () => {
    const response = await fetch('/api/v1/admin/sessions/stats');
    const data: SessionStats = await response.json();
    setSessionStats(data);
  };

  React.useEffect(() => {
    fetchSessionStats();
  }, []);

  if (!sessionStats) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h1>Sessions Management</h1>
      <div className="stat-cards">
        <div className="stat-card">
          <h2>Total Sessions</h2>
          <p>{sessionStats.totalSessions}</p>
        </div>
        <div className="stat-card">
          <h2>Active Sessions</h2>
          <p>{sessionStats.activeSessions}</p>
        </div>
        <div className="stat-card">
          <h2>Total Messages</h2>
          <p>{sessionStats.totalMessages}</p>
        </div>
        <div className="stat-card">
          <h2>Total Tokens</h2>
          <p>{sessionStats.totalTokens}</p>
        </div>
        <div className="stat-card">
          <h2>Sessions Today</h2>
          <p>{sessionStats.sessionsToday}</p>
        </div>
        <div className="stat-card">
          <h2>Average Messages Per Session</h2>
          <p>{sessionStats.averageMessagesPerSession}</p>
        </div>
        <div className="stat-card">
          <h2>Sessions This Week</h2>
          <p>{sessionStats.sessionsThisWeek}</p>
        </div>
      </div>
    </div>
  );
};

export default AdminSessions;
```

This code adds two new stat cards for `averageMessagesPerSession` and `sessionsThisWeek`, which will display the corresponding values from the `SessionStats` object.