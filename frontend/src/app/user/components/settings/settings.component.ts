import {Component, OnInit} from '@angular/core';
import {FormBuilder, FormGroup, ValidationErrors, ValidatorFn, Validators} from '@angular/forms';
import {UserViewModelService} from '../../services/user-view-model.service';
import {MainComponent} from '../../../shared/components/main/main.component';
import {finalize} from 'rxjs/operators';
import {MainService} from '../../../core/services/main.service';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.scss']
})
export class SettingsComponent extends MainComponent implements OnInit {

  formGroup: FormGroup;

  constructor(
    private viewModel: UserViewModelService,
    private formBuilder: FormBuilder,
    private service: MainService,
  ) {
    super();

    this.formGroup = formBuilder.group({
      password: formBuilder.control('', [Validators.required, Validators.minLength(8), Validators.maxLength(128)]),
      passwordConfirm: formBuilder.control('', [Validators.required, passwordMatchValidator])
    });
  }

  ngOnInit(): void {
  }

  updatePassword(value: any): void {
    this.enterProcessingState();

    this.subs.add(
      this.viewModel.updatePassword(value.password).pipe(
        finalize(() => this.leaveProcessingState())
      ).subscribe(_ => {
        this.service.alertSuccess('Password is updated.');
        this.formGroup.reset();
      })
    );
  }
}

const passwordMatchValidator: ValidatorFn = (control: FormGroup): ValidationErrors | null => {
  if (!control.parent) {
    return null;
  }

  const password = control.parent.get('password');

  return control.value !== password.value ? {passwordMismatch: true} : null;
};
