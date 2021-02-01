import {Injectable} from '@angular/core';
import {BehaviorSubject, Observable, Subject, Subscription, timer} from 'rxjs';
import {ConnectionStatus, IncomingMessageKind, NotificationKind, OutgoingMessageKind} from '../websocket/models';
import {environment} from '../../../environments/environment';
import {map, tap} from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class WebSocketService {
  private connection: WebSocket;
  // Connection related
  // setInterval number. In order to cancel interval, we store it
  private interval: number;

  private willinglyClosed = false;

  private $status = new BehaviorSubject<ConnectionStatus>(ConnectionStatus.Initial);

  private $willConnectIn = new Subject<number>();

  private timer: Subscription;
  // in seconds
  private timings = [
    0, 15, 30, 75, 120
  ];

  private activeTimingIndex = 0;

  // Notification related
  private $notification = new Subject<any>();

  constructor() {
    this.connect();
  }

  connect(): void {
    this.willinglyClosed = false;

    // If we have already connected or reconnecting, do not try anything
    if (this.$status.value === ConnectionStatus.Open || this.$status.value === ConnectionStatus.Reconnecting) {
      throw Error('Already connected or reconnecting');
    }

    // If status is in initial state, do not send any Reconnecting message
    if (this.$status.value !== ConnectionStatus.Initial) {
      this.$status.next(ConnectionStatus.Reconnecting);
    }

    // If someone wants to connect now and there is a timer, cancel the timer
    if (this.timer) {
      this.timer.unsubscribe();
    }

    this.connection = new WebSocket(environment.wsEndpoint + `?token=${localStorage.getItem('token')}`);

    this.connection.onmessage = (event) => this.messageHandler(event);

    this.connection.onopen = () => {
      // Reset the timing index
      this.activeTimingIndex = 0;

      this.$status.next(ConnectionStatus.Open);

      // Send ping in 30 seconds interval
      this.interval = setInterval(() => {
        this.send({
          kind: OutgoingMessageKind.KeepAlive
        });
      }, 1000 * 30);
    };

    this.connection.onerror = (e) => console.log('error', e);

    this.connection.onclose = (_) => {
      // If we did not close the connection by calling disconnect fn, set a timer on whose ending will cause a reconnect
      if (!this.willinglyClosed) {
        this.timer = timer(1000, 1000).pipe(
          map(t => this.timings[this.activeTimingIndex] - t - 1),
          tap(t => {
            if (t <= 0) {
              this.connect();
            }
          })
        ).subscribe(t => this.$willConnectIn.next(t));
      }

      // Increase the index in order to try reconnecting after much more longer than previous wait time
      if (this.activeTimingIndex < this.timings.length - 1) {
        this.activeTimingIndex += 1;
      }

      this.$status.next(ConnectionStatus.Closed);

      // If there is interval, clear it since connection is closed
      if (this.interval) {
        clearInterval(this.interval);
        this.interval = null;
      }
    };
  }

  disconnect(): void {
    this.willinglyClosed = true;

    this.connection.close();

    // Do not update status since connection.onclose already does that for us
  }

  listenNotifications(): Subject<{ userId: number; message: { kind: NotificationKind; data: any; } }> {
    return this.$notification;
  }

  listenConnectionStatus(): BehaviorSubject<ConnectionStatus> {
    return this.$status;
  }

  willReconnectIn(): Observable<number> {
    return this.$willConnectIn;
  }

  private messageHandler(event: MessageEvent): void {
    const message = JSON.parse(event.data);

    if (message.kind === IncomingMessageKind.Notification) {
      this.$notification.next(message.data);
    } else if (message.kind === IncomingMessageKind.Error) {
      console.error(message.data);
    }
  }

  private send(obj: any): void {
    this.connection.send(JSON.stringify(obj));
  }
}
