import {Component, OnInit} from '@angular/core';
import {AuthViewModelService} from '../../services/auth-view-model.service';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ErrorMessage} from '../../../core/models';
import {catchError, finalize} from 'rxjs/operators';
import {Router} from '@angular/router';
import {WebSocketService} from '../../../core/services/web-socket.service';

@Component({
  selector: 'app-login',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.scss']
})
export class LoginComponent extends MainComponent implements OnInit {

  errorMessage: ErrorMessage;

  formGroup: FormGroup;

  constructor(
    private viewModel: AuthViewModelService,
    private formBuilder: FormBuilder,
    private router: Router,
    private webSocketService: WebSocketService
  ) {
    super();

    this.formGroup = formBuilder.group({
      email: formBuilder.control('', [Validators.required, Validators.email]),
      password: formBuilder.control('', [Validators.required])
    });
  }

  ngOnInit(): void {
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
        try {
          this.webSocketService.connect();
        } catch (e) {
          console.error(e);
        }

        return this.router.navigate(['/']);
      })
    );
  }
}
