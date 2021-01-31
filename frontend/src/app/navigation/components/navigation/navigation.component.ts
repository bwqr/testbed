import {Component, OnInit} from '@angular/core';
import {MainService} from '../../../core/services/main.service';
import {Alert} from '../../../core/models';

@Component({
  selector: 'app-navigation',
  templateUrl: './navigation.component.html',
  styleUrls: ['./navigation.component.scss']
})
export class NavigationComponent implements OnInit {

  alerts: Alert[] = [];

  constructor(
    private service: MainService
  ) {
    this.service.listenAlerts().subscribe(alert => this.alerts.push(alert));
  }

  ngOnInit(): void {
  }

  toggleNavbar(navbarId: string): void {
    const target = document.getElementById(navbarId);

    if (target) {
      target.classList.toggle('show');
    }
  }

  removeAlert(alert: Alert): void {
    const index = this.alerts.findIndex(a => a === alert);

    if (index !== -1) {
      this.alerts.splice(index, 1);
    }
  }
}
