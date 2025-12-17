import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App, { AppToaster } from "./app";
import { AuthProvider } from "./lib/router";
import "./app.css";

ReactDOM.createRoot(document.getElementById(`root`)!).render(
    <React.StrictMode>
        <BrowserRouter>
            <AuthProvider>
                <App />
                <AppToaster />
            </AuthProvider>
        </BrowserRouter>
    </React.StrictMode>
);
