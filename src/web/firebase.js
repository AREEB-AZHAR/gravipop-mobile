import { initializeApp } from "firebase/app";
import { getAnalytics, isSupported } from "firebase/analytics";

// GraviPop Web App Firebase configuration
export const firebaseConfig = {
  apiKey: "AIzaSyB_po2OjopKvtGzS2onFhBLRWOE6qAYnn4",
  authDomain: "gravipop-mobile.firebaseapp.com",
  projectId: "gravipop-mobile",
  storageBucket: "gravipop-mobile.firebasestorage.app",
  messagingSenderId: "763464770635",
  appId: "1:763464770635:web:272b5f4de49c2682c559ce",
  measurementId: "G-EHP5QCL9NY"
};

// Initialize Firebase App
export const app = initializeApp(firebaseConfig);

// Initialize Firebase Analytics if supported
export let analytics = null;
if (typeof window !== "undefined") {
  isSupported()
    .then((supported) => {
      if (supported) {
        analytics = getAnalytics(app);
      }
    })
    .catch(() => {});
}
