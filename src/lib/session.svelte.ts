import { initialState, reduce, type SessionAction, type SessionState } from './session-reducer';

class SessionStore {
  state = $state<SessionState>(initialState);

  dispatch(action: SessionAction): void {
    this.state = reduce(this.state, action);
  }
}

export const session = new SessionStore();
