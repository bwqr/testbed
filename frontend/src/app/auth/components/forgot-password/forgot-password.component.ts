import {Component, OnInit} from '@angular/core';
import {ErrorMessage} from '../../../core/models';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {AuthViewModelService} from '../../services/auth-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';
import {catchError, finalize} from 'rxjs/operators';
import {locales} from '../../../locales';

@Component({
  selector: 'app-forgot-password',
  templateUrl: './forgot-password.component.html',
  styleUrls: ['./forgot-password.component.scss']
})
export class ForgotPasswordComponent extends MainComponent implements OnInit {

  formGroup: FormGroup;

  isSuccessful = false;

  errorMessage: ErrorMessage;

  constructor(
    private viewModel: AuthViewModelService,
    private formBuilder: FormBuilder,
  ) {
    super();

    this.formGroup = this.formBuilder.group({
      email: this.formBuilder.control('', [Validators.required, Validators.email])
    });
  }

  ngOnInit(): void {
  }

  forgotPassword(value: any): void {
    this.enterProcessingState();
    this.errorMessage = null;

    this.subs.add(
      this.viewModel.forgotPassword(value.email).pipe(
        finalize(() => this.leaveProcessingState()),
        catchError(errorMessage => {
          if (errorMessage instanceof ErrorMessage) {
            this.errorMessage = errorMessage;

            // ItemNotFound Error
            if (errorMessage.errorCode === 101) {
              this.errorMessage.message = {
                key: errorMessage.message.key,
                localized: locales.auth.errors.emailNotFound
              };
            }
          }
          return Promise.reject(errorMessage);
        })
      ).subscribe(_ => this.isSuccessful = true)
    );
  }
}
