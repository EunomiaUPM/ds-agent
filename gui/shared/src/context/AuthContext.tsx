import { createContext, ReactNode, useEffect, useState } from "react";
import { ParticipantDto } from "../data/orval/model";

export interface AuthContextType {
  isAuthenticated: boolean;
  participant: ParticipantDto | null;
  clientToken: string | null;
  setAuthentication: (participant: ParticipantDto, clientToken: string) => void;
  unsetAuthentication: () => void;
}

interface AuthStorageData {
  participant: ParticipantDto | null;
  clientToken: string | null;
}

const AUTH_STORAGE_KEY = "auth_data";

export const AuthContext = createContext<AuthContextType | null>(null);

export const AuthContextProvider = ({ children }: { children: ReactNode }) => {
  /**
   * Load initial auth state
   */
  const loadInitialAuthState = (): AuthStorageData => {
    try {
      const storedData = localStorage.getItem(AUTH_STORAGE_KEY);
      if (storedData) {
        return JSON.parse(storedData);
      }
    } catch (error) {
      console.error("Error parsing auth data from localStorage:", error);
      localStorage.removeItem(AUTH_STORAGE_KEY);
    }
    return { participant: null, clientToken: null };
  };

  /**
   * Inits auth state from localstorage
   */
  const initialAuthData = loadInitialAuthState();

  /**
   * State
   */
  const [participant, setParticipant] = useState<ParticipantDto | null>(initialAuthData.participant);
  const [clientToken, setClientToken] = useState<string | null>(initialAuthData.clientToken);
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>(
    !!initialAuthData.participant && !!initialAuthData.clientToken,
  );

  /**
   * Coordinate auth state and localstorage
   */
  useEffect(() => {
    if (participant && clientToken) {
      const dataToStore: AuthStorageData = { participant, clientToken };
      localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(dataToStore));
      setIsAuthenticated(true); // Asegúrate de que isAuthenticated sea true
    } else {
      localStorage.removeItem(AUTH_STORAGE_KEY);
      setIsAuthenticated(false); // Asegúrate de que isAuthenticated sea false
    }
  }, [participant, clientToken]);

  /**
   * Outer interface
   */
  const setAuthentication = (participant: ParticipantDto, clientToken: string) => {
    setParticipant(participant);
    setClientToken(clientToken);
  };
  const unsetAuthentication = () => {
    setParticipant(null);
    setClientToken(null);
  };

  const value = {
    isAuthenticated,
    participant,
    clientToken,
    setAuthentication,
    unsetAuthentication,
  };
  // @ts-ignore
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
