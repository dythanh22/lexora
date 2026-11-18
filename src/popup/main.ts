import { mount } from 'svelte';
import '../styles/tokens.css';
import Popup from './Popup.svelte';

mount(Popup, { target: document.getElementById('app')! });
