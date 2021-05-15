import {Component, OnInit} from '@angular/core';
import {ErrorMessage, RetryRequest} from '../../../core/models';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {RequestFailService} from '../../../core/services/request-fail.service';
import {AuthViewModelService} from '../../services/auth-view-model.service';
import {UserViewModelService} from '../../../user/services/user-view-model.service';
import {Router} from '@angular/router';
import {MainComponent} from '../../../shared/components/main/main.component';
import {WebSocketService} from '../../../core/services/web-socket.service';
import {catchError, finalize} from 'rxjs/operators';

@Component({
  selector: 'app-login-dialog',
  templateUrl: './login-dialog.component.html',
  styleUrls: ['./login-dialog.component.scss']
})
export class LoginDialogComponent extends MainComponent implements OnInit {
  retryRequests: Array<RetryRequest> = [];

  errorMessage: ErrorMessage;

  formGroup: FormGroup;

  constructor(
    private requestFailService: RequestFailService,
    private formBuilder: FormBuilder,
    private viewModel: AuthViewModelService,
    private userViewModel: UserViewModelService,
    private router: Router,
    private webSocketService: WebSocketService
  ) {
    super();

    this.subs.add(this.requestFailService.failedRequests.subscribe(request => this.retryRequests.push(request)));

    this.formGroup = formBuilder.group({
      email: formBuilder.control('', [Validators.required, Validators.email]),
      password: formBuilder.control('', [Validators.required])
    });
  }

  ngOnInit(): void {
    this.webSocketService.disconnect();
  }

  login(value: any): void {
    this.errorMessage = null;
    this.enterProcessingState();

    this.subs.add(
      this.viewModel.login(value.email, value.password).pipe(
        catchError(error => {
          if (error instanceof ErrorMessage) {
            this.errorMessage = error;
          }

          return Promise.reject(error);
        }),
        finalize(() => this.leaveProcessingState())
      ).subscribe(_ => {
        // Retry failed Requests
        for (const req of this.retryRequests) {
          this.requestFailService.retryFailedRequests.next(req);
        }

        try {
          this.webSocketService.connect();
        } catch (e) {
          console.error(e);
        }

        return this.router.navigate([{outlets: {auth: null}}]);
      })
    );
  }
}
