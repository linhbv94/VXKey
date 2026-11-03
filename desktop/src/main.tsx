import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './app';
import './index.css';
import { initialSurface } from './services/window_surface';
document.documentElement.dataset.surface = initialSurface();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
