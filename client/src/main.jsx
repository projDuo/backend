import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.jsx'
import ElementsBg from './components/ElementsBg/ElementsBg.jsx'
import { AuthProvider } from './context/AuthContext.jsx'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <ElementsBg />
    <AuthProvider>
      <App />
    </AuthProvider>
  </StrictMode>,
)
