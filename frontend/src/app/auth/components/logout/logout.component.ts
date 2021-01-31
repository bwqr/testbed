import {Component, OnInit} from '@angular/core';
import {AuthViewModelService} from '../../services/auth-view-model.service';
import {Router} from '@angular/router';

@Component({
  selector: 'app-logout',
  templateUrl: './logout.component.html',
  styleUrls: ['./logout.component.scss']
})
export class LogoutComponent implements OnInit {

  constructor(
    private viewModel: AuthViewModelService,
    private router: Router,
  ) {
  }

  ngOnInit(): void {
    this.viewModel.logout();

    this.router.navigate(['/auth/login']).then();
  }

}
