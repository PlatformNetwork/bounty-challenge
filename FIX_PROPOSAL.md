To fix the issue of hardcoded English strings in the `CortexChangesPanel` component, we need to integrate it with the IDE's i18n mechanism. We can achieve this by using the `I18nContext` from `src/i18n/index.ts` to translate the strings.

Here's the exact code fix:

```tsx
// src/components/cortex/CortexChangesPanel.tsx
import React from 'react';
import { useI18n } from 'src/i18n/index';

const CortexChangesPanel = () => {
  const { t } = useI18n();

  return (
    // ...
    <div>
      <div>{t('cortex.changesPanel.tabLabels.changes')}</div>
      <div>{t('cortex.changesPanel.tabLabels.allFiles')}</div>
      <div>{t('cortex.changesPanel.emptyState')}</div>
      <div>{t('cortex.changesPanel.diffLoadingFallback')}</div>
    </div>
    // ...
  );
};

export default CortexChangesPanel;
```

Then, in your `src/i18n/index.ts` file, add the necessary translation keys:

```typescript
// src/i18n/index.ts
import { createI18n } from 'i18n';

const i18n = createI18n({
  // ...
  resources: {
    en: {
      translation: {
        cortex: {
          changesPanel: {
            tabLabels: {
              changes: 'Changes',
              allFiles: 'All Files',
            },
            emptyState: 'No changes yet',
            diffLoadingFallback: 'Unable to load diff',
          },
        },
      },
    },
    // Add translations for other languages here
    fr: {
      translation: {
        cortex: {
          changesPanel: {
            tabLabels: {
              changes: 'Changements',
              allFiles: 'Tous les fichiers',
            },
            emptyState: 'Aucun changement pour le moment',
            diffLoadingFallback: 'Impossible de charger la différence',
          },
        },
      },
    },
    // ...
  },
});

export const useI18n = () => {
  const { t } = i18n.useTranslation();
  return { t };
};
```

This code uses the `useI18n` hook to get the `t` function, which is used to translate the strings. The `t` function takes a key as an argument and returns the translated string for the current locale.

Make sure to add translations for all the languages you want to support in the `resources` object.