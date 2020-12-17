import {Component, OnInit} from '@angular/core';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {AuthViewModelService} from '../../services/auth-view-model.service';
import {Router} from '@angular/router';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ErrorMessage} from '../../../core/models';
import {catchError, finalize} from 'rxjs/operators';

@Component({
  selector: 'app-sign-up',
  templateUrl: './sign-up.component.html',
  styleUrls: ['./sign-up.component.scss']
})
export class SignUpComponent extends MainComponent implements OnInit {

  errorMessage: ErrorMessage;

  formGroup: FormGroup;

  constructor(
    private viewModel: AuthViewModelService,
    private formBuilder: FormBuilder,
    private router: Router,
  ) {
    super();

    this.formGroup = formBuilder.group({
      firstName: formBuilder.control('', [Validators.required]),
      lastName: formBuilder.control('', [Validators.required]),
      email: formBuilder.control('', [Validators.required, Validators.email]),
      password: formBuilder.control('', [Validators.required]),
    });
  }

  ngOnInit(): void {
  }

  signUp(value: any): void {
    this.enterProcessingState();

    this.subs.add(
      this.viewModel.signUp(value.firstName, value.lastName, value.email, value.password).pipe(
        catchError(error => {
          if (error instanceof ErrorMessage) {
            this.errorMessage = error;
          }

          return Promise.reject(error);
        }),
        finalize(() => this.leaveProcessingState())
      ).subscribe(_ => this.router.navigate(['/login']))
    );
  }
}
